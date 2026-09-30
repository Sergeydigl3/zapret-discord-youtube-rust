//! Real QUIC v1 Initial packet construction (RFC 9000) with proper
//! TLS 1.3 ClientHello (RFC 9001) protection.
//!
//! Network filters that only drop "fake" QUIC probes (arbitrary long-header
//! datagrams) but pass real browser HTTP/3 traffic can only be detected by
//! sending a genuine Initial packet. This module builds one, verifies it
//! against the RFC 9001 Appendix A.2 test vector, and exposes a UDP probe.

use std::io;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockEncrypt, KeyInit as AesKeyInit};
use aes_gcm::aead::{Aead, Payload};

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;
type Aes128 = aes::Aes128;
type Aes128Gcm = aes_gcm::Aes128Gcm;

/// RFC 9001 section 5.2 initial_salt.
const INITIAL_SALT: [u8; 20] = [
    0x38, 0x76, 0x2c, 0xf7, 0xf5, 0x59, 0x34, 0xb3, 0x4d, 0x17, 0x9a, 0xe6, 0xa4, 0xc8, 0x0c, 0xad, 0xcc, 0xbb, 0x7f,
    0x0a,
];

const CLIENT_IN_LABEL: &str = "client in";
const QUIC_KEY_LABEL: &str = "quic key";
const QUIC_IV_LABEL: &str = "quic iv";
const QUIC_HP_LABEL: &str = "quic hp";
const TLS13_PREFIX: &[u8] = b"tls13 ";

/// Servers MUST discard Initial datagrams smaller than 1200 bytes (RFC 9000
/// section 14.1). Datagrams larger than that are also dropped (or ICMP
/// reduced), so the probe is padded to exactly 1200 bytes.
const TARGET_DATAGRAM: usize = 1200;

/// Public key of X25519 base point (u=9), little-endian.
const X25519_BASE_PUBLIC: [u8; 32] = {
    let mut k = [0u8; 32];
    k[0] = 9;
    k
};

#[derive(Debug, Clone, PartialEq)]
pub struct InitialKeys {
    pub key: [u8; 16],
    pub iv: [u8; 12],
    pub hp: [u8; 16],
}

fn hkdf_extract(salt: &[u8], ikm: &[u8]) -> [u8; 32] {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(salt).expect("HMAC accepts any key length");
    mac.update(ikm);
    mac.finalize().into_bytes().into()
}

fn hkdf_expand(prk: &[u8], info: &[u8], out_len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(out_len);
    let mut t = Vec::new();
    let mut counter: u8 = 1;
    while out.len() < out_len {
        let mut mac = <HmacSha256 as Mac>::new_from_slice(prk).expect("HMAC accepts any key length");
        mac.update(&t);
        mac.update(info);
        mac.update(&[counter]);
        t = mac.finalize().into_bytes().to_vec();
        out.extend_from_slice(&t);
        counter += 1;
    }
    out.truncate(out_len);
    out
}

/// TLS 1.3 HKDF-Expand-Label (RFC 8446 section 7.1).
fn hkdf_expand_label(secret: &[u8], label: &str, context: &[u8], out_len: usize) -> Vec<u8> {
    let mut full_label = Vec::with_capacity(TLS13_PREFIX.len() + label.len());
    full_label.extend_from_slice(TLS13_PREFIX);
    full_label.extend_from_slice(label.as_bytes());

    let mut info = Vec::new();
    info.extend_from_slice(&(out_len as u16).to_be_bytes());
    info.push(full_label.len() as u8);
    info.extend_from_slice(&full_label);
    info.push(context.len() as u8);
    info.extend_from_slice(context);
    hkdf_expand(secret, &info, out_len)
}

/// Derives the client Initial packet protection keys from the Destination
/// Connection ID (RFC 9001 sections 5.1 and 5.2).
pub fn derive_initial_keys(dcid: &[u8]) -> InitialKeys {
    let initial_secret = hkdf_extract(&INITIAL_SALT, dcid);
    let client_secret = hkdf_expand_label(&initial_secret, CLIENT_IN_LABEL, b"", 32);
    let key = hkdf_expand_label(&client_secret, QUIC_KEY_LABEL, b"", 16);
    let iv = hkdf_expand_label(&client_secret, QUIC_IV_LABEL, b"", 12);
    let hp = hkdf_expand_label(&client_secret, QUIC_HP_LABEL, b"", 16);
    InitialKeys {
        key: key.try_into().expect("16 bytes"),
        iv: iv.try_into().expect("12 bytes"),
        hp: hp.try_into().expect("16 bytes"),
    }
}

fn aes_ecb(key: &[u8; 16], block: &[u8]) -> [u8; 16] {
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut out = GenericArray::clone_from_slice(block);
    cipher.encrypt_block(&mut out);
    out.into()
}

/// RFC 9001 section 5.4.2 header protection for short packet numbers.
fn apply_header_protection(pkt: &mut [u8], pn_offset: usize, pn_len: usize, hp: &[u8; 16]) {
    let sample_offset = pn_offset + 4;
    let sample = &pkt[sample_offset..sample_offset + 16];
    let mask = aes_ecb(hp, sample);
    pkt[0] ^= mask[0] & 0x0f;
    for i in 0..pn_len {
        pkt[pn_offset + i] ^= mask[1 + i];
    }
}

/// QUIC variable-length integer (RFC 9000 section 16).
fn varint(mut value: u64) -> Vec<u8> {
    let mut prefix = 0u8;
    let mut len = 1;
    while value >= (1u64 << (6 + 8 * (len - 1))) && len < 8 {
        len <<= 1;
        prefix += 1;
    }
    let mut out = vec![0u8; len];
    for i in (0..len).rev() {
        out[i] = (value & 0xff) as u8;
        value >>= 8;
    }
    out[0] |= prefix << 6;
    out
}

fn encode_pn(pn: u64, len: usize) -> Vec<u8> {
    let be = pn.to_be_bytes();
    be[be.len() - len..].to_vec()
}

/// XORs the 12-byte IV with the packet number right-aligned to 12 bytes
/// (RFC 9001 section 5.3).
fn build_nonce(iv: &[u8; 12], pn: u64) -> [u8; 12] {
    let mut nonce = *iv;
    let pnb = pn.to_be_bytes();
    for i in 0..8 {
        nonce[4 + i] ^= pnb[i];
    }
    nonce
}

fn aes128gcm_encrypt(key: &[u8; 16], nonce: &[u8; 12], aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes128Gcm::new(GenericArray::from_slice(key));
    let payload = Payload { msg: plaintext, aad };
    cipher
        .encrypt(GenericArray::from_slice(nonce), payload)
        .expect("AES-128-GCM encryption failed")
}

/// Builds a protected QUIC v1 Initial packet (RFC 9000 section 17.2.1).
/// `frames` are the unprotected payload frames; the AEAD tag is appended by
/// the cipher. `pn_len` selects the packet number field size (1, 2, 3 or 4).
pub fn build_initial_packet(dcid: &[u8], scid: &[u8], pn: u64, pn_len: usize, frames: &[u8]) -> Vec<u8> {
    debug_assert!((1..=4).contains(&pn_len));
    let keys = derive_initial_keys(dcid);

    let mut header = Vec::new();
    header.push(0xc0 | ((pn_len - 1) as u8));
    header.extend_from_slice(&1u32.to_be_bytes()); // QUIC version 1
    header.push(dcid.len() as u8);
    header.extend_from_slice(dcid);
    header.push(scid.len() as u8);
    header.extend_from_slice(scid);
    header.extend_from_slice(&varint(0)); // token length (no token)

    let length = (pn_len + frames.len() + 16) as u64; // PN + payload + tag
    header.extend_from_slice(&varint(length));

    let pn_offset = header.len();
    header.extend_from_slice(&encode_pn(pn, pn_len));

    let aad = header.clone();
    let nonce = build_nonce(&keys.iv, pn);
    let ciphertext = aes128gcm_encrypt(&keys.key, &nonce, &aad, frames);
    header.extend_from_slice(&ciphertext);

    apply_header_protection(&mut header, pn_offset, pn_len, &keys.hp);
    header
}

// ---------------------------------------------------------------------------
// TLS 1.3 ClientHello construction
// ---------------------------------------------------------------------------

fn push_ext(out: &mut Vec<u8>, ext_type: u16, data: &[u8]) {
    out.extend_from_slice(&ext_type.to_be_bytes());
    out.extend_from_slice(&(data.len() as u16).to_be_bytes());
    out.extend_from_slice(data);
}

fn ext_server_name(out: &mut Vec<u8>, name: &[u8]) {
    let mut data = Vec::new();
    data.extend_from_slice(&(1 + 2 + name.len() as u16).to_be_bytes());
    data.push(0); // host_name
    data.extend_from_slice(&(name.len() as u16).to_be_bytes());
    data.extend_from_slice(name);
    push_ext(out, 0x0000, &data);
}

fn ext_supported_groups(out: &mut Vec<u8>) {
    let groups = [0x00, 0x1d, 0x00, 0x17]; // x25519, secp256r1
    let mut data = Vec::new();
    data.extend_from_slice(&(groups.len() as u16).to_be_bytes());
    data.extend_from_slice(&groups);
    push_ext(out, 0x000a, &data);
}

fn ext_signature_algorithms(out: &mut Vec<u8>) {
    let algs = [0x04, 0x03, 0x08, 0x04, 0x04, 0x01];
    let mut data = Vec::new();
    data.extend_from_slice(&(algs.len() as u16).to_be_bytes());
    data.extend_from_slice(&algs);
    push_ext(out, 0x000d, &data);
}

fn ext_alpn(out: &mut Vec<u8>, protocols: &[&[u8]]) {
    let mut list = Vec::new();
    for p in protocols {
        list.push(p.len() as u8);
        list.extend_from_slice(p);
    }
    let mut data = Vec::new();
    data.extend_from_slice(&(list.len() as u16).to_be_bytes());
    data.extend_from_slice(&list);
    push_ext(out, 0x0010, &data);
}

fn ext_key_share(out: &mut Vec<u8>) {
    let mut data = Vec::new();
    data.extend_from_slice(&36u16.to_be_bytes()); // 2 + 2 + 32
    data.extend_from_slice(&[0x00, 0x1d]); // x25519
    data.extend_from_slice(&32u16.to_be_bytes());
    data.extend_from_slice(&X25519_BASE_PUBLIC);
    push_ext(out, 0x0033, &data);
}

fn ext_supported_versions(out: &mut Vec<u8>) {
    let mut data = Vec::new();
    data.push(2);
    data.extend_from_slice(&[0x03, 0x04]); // TLS 1.3
    push_ext(out, 0x002b, &data);
}

fn ext_quic_transport_parameters(out: &mut Vec<u8>, scid: &[u8]) {
    let mut data = Vec::new();
    data.push(0x0f); // initial_source_connection_id (0x0f)
    data.extend_from_slice(&varint(scid.len() as u64));
    data.extend_from_slice(scid);
    push_ext(out, 0x0039, &data);
}

/// Builds a TLS 1.3 ClientHello handshake message (RFC 8446) with the
/// extensions required by a QUIC client (RFC 9001 section 8.1).
pub fn build_client_hello(server_name: &str, scid: &[u8]) -> Vec<u8> {
    let mut extensions = Vec::new();
    ext_server_name(&mut extensions, server_name.as_bytes());
    ext_supported_groups(&mut extensions);
    ext_signature_algorithms(&mut extensions);
    ext_alpn(&mut extensions, &[b"h3"]);
    ext_key_share(&mut extensions);
    ext_supported_versions(&mut extensions);
    ext_quic_transport_parameters(&mut extensions, scid);

    let random = random_bytes(32);

    let mut body = Vec::new();
    body.extend_from_slice(&[0x03, 0x03]); // legacy_version TLS 1.2
    body.extend_from_slice(&random);
    body.push(32);
    body.extend_from_slice(&[0u8; 32]); // legacy_session_id
    body.extend_from_slice(&6u16.to_be_bytes());
    body.extend_from_slice(&[0x13, 0x01, 0x13, 0x02, 0x13, 0x03]);
    body.push(1);
    body.push(0); // compression_methods: null
    body.extend_from_slice(&(extensions.len() as u16).to_be_bytes());
    body.extend_from_slice(&extensions);

    let mut msg = Vec::new();
    msg.push(0x01); // ClientHello
    msg.extend_from_slice(&[(body.len() >> 16) as u8, (body.len() >> 8) as u8, body.len() as u8]);
    msg.extend_from_slice(&body);
    msg
}

/// Builds the CRYPTO frame carrying the ClientHello, followed by PADDING
/// frames to reach the required Initial datagram size.
fn build_initial_frames(ch: &[u8]) -> Vec<u8> {
    let mut frames = Vec::new();
    frames.push(0x06); // CRYPTO
    frames.extend_from_slice(&varint(0)); // offset
    frames.extend_from_slice(&varint(ch.len() as u64));
    frames.extend_from_slice(ch);
    frames
}

// ---------------------------------------------------------------------------
// Probe
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProbeOutcome {
    Reply,
    NoReply,
    Error,
}

fn send_one_probe(sock: &std::net::UdpSocket, server_name: &str) -> ProbeOutcome {
    let dcid = random_bytes(8);
    let scid = random_bytes(8);
    let ch = build_client_hello(server_name, &scid);
    let mut frames = build_initial_frames(&ch);

    // Header size without the PN field, length field and AEAD tag: first byte,
    // version, DCID len+id, SCID len+id, token len. With an 8-byte DCID and
    // SCID and a 2-byte length varint this is 26 bytes.
    let fixed_header = 1 + 4 + 1 + dcid.len() + 1 + scid.len() + 1;
    let pn_len = 1;
    let length_varint_len = 2;
    let tag_len = 16;
    let max_frames = TARGET_DATAGRAM.saturating_sub(fixed_header + length_varint_len + pn_len + tag_len);
    if frames.len() < max_frames {
        frames.resize(max_frames, 0x00); // PADDING frames
    }

    let pkt = build_initial_packet(&dcid, &scid, 0, pn_len, &frames);
    if sock.send(&pkt).is_err() {
        return ProbeOutcome::Error;
    }

    let mut buf = [0u8; 2048];
    match sock.recv(&mut buf) {
        Ok(n) if n > 0 => ProbeOutcome::Reply,
        Ok(_) => ProbeOutcome::NoReply,
        Err(ref e) if e.kind() == io::ErrorKind::TimedOut || e.kind() == io::ErrorKind::WouldBlock => {
            ProbeOutcome::NoReply
        }
        Err(_) => ProbeOutcome::Error,
    }
}

/// Sends one genuine QUIC v1 Initial packet on an already connected socket and
/// reports whether a reply arrived (using the socket's read timeout).
pub fn send_probe(sock: &std::net::UdpSocket, server_name: &str) -> ProbeOutcome {
    send_one_probe(sock, server_name)
}

/// Sends genuine QUIC v1 Initial packets to `addr` and reports whether any
/// reply arrives. Returns true on the first response; false if all attempts
/// time out; also false on socket errors.
pub fn probe_quic(addr: SocketAddr, server_name: &str, attempts: usize, timeout: Duration) -> bool {
    let sock = match std::net::UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return false,
    };
    if sock.connect(addr).is_err() {
        return false;
    }
    if sock.set_read_timeout(Some(timeout)).is_err() {
        return false;
    }
    for _ in 0..attempts {
        match send_one_probe(&sock, server_name) {
            ProbeOutcome::Reply => return true,
            ProbeOutcome::NoReply => continue,
            ProbeOutcome::Error => return false,
        }
    }
    false
}

fn random_bytes(n: usize) -> Vec<u8> {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9e3779b97f4a7c15)
        ^ 0x9e3779b97f4a7c15;
    let mut state = if seed == 0 { 0x9e3779b97f4a7c15 } else { seed };
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        out.push(state as u8);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const INITIAL_SECRET: &str = "7db5df06e7a69e432496adedb00851923595221596ae2ae9fb8115c1e9ed0a44";
    const CLIENT_INITIAL_SECRET: &str = "c00cf151ca5be075ed0ebfb5c80323c42d6b7db67881289af4008f1f6c357aea";
    const CLIENT_INITIAL_KEY: &str = "1f369613dd76d5467730efcbe3b1a22d";
    const CLIENT_INITIAL_IV: &str = "fa044b2f42a3fd3b46fb255c";
    const CLIENT_INITIAL_HP: &str = "9f50449e04a0e810283a1e9933adedd2";

    const A2_PAYLOAD: &str = "060040f1010000ed0303ebf8fa56f12939b9584a3896472ec40bb863cfd3e86804fe3a47f06a2b69484c00000413011302010000c000000010000e00000b6578616d706c652e636f6dff01000100000a00080006001d0017001800100007000504616c706e000500050100000000003300260024001d00209370b2c9caa47fbabaf4559fedba753de171fa71f50f1ce15d43e994ec74d748002b0003020304000d0010000e0403050306030203080408050806002d00020101001c00024001003900320408ffffffffffffffff05048000ffff07048000ffff0801100104800075300901100f088394c8f03e51570806048000ffff";

    const A2_PACKET: &str = "c000000001088394c8f03e5157080000449e7b9aec34d1b1c98dd7689fb8ec11d242b123dc9bd8bab936b47d92ec356c0bab7df5976d27cd449f63300099f3991c260ec4c60d17b31f8429157bb35a1282a643a8d2262cad67500cadb8e7378c8eb7539ec4d4905fed1bee1fc8aafba17c750e2c7ace01e6005f80fcb7df621230c83711b39343fa028cea7f7fb5ff89eac2308249a02252155e2347b63d58c5457afd84d05dfffdb20392844ae812154682e9cf012f9021a6f0be17ddd0c2084dce25ff9b06cde535d0f920a2db1bf362c23e596d11a4f5a6cf3948838a3aec4e15daf8500a6ef69ec4e3feb6b1d98e610ac8b7ec3faf6ad760b7bad1db4ba3485e8a94dc250ae3fdb41ed15fb6a8e5eba0fc3dd60bc8e30c5c4287e53805db059ae0648db2f64264ed5e39be2e20d82df566da8dd5998ccabdae053060ae6c7b4378e846d29f37ed7b4ea9ec5d82e7961b7f25a9323851f681d582363aa5f89937f5a67258bf63ad6f1a0b1d96dbd4faddfcefc5266ba6611722395c906556be52afe3f565636ad1b17d508b73d8743eeb524be22b3dcbc2c7468d54119c7468449a13d8e3b95811a198f3491de3e7fe942b330407abf82a4ed7c1b311663ac69890f4157015853d91e923037c227a33cdd5ec281ca3f79c44546b9d90ca00f064c99e3dd97911d39fe9c5d0b23a229a234cb36186c4819e8b9c5927726632291d6a418211cc2962e20fe47feb3edf330f2c603a9d48c0fcb5699dbfe5896425c5bac4aee82e57a85aaf4e2513e4f05796b07ba2ee47d80506f8d2c25e50fd14de71e6c418559302f939b0e1abd576f279c4b2e0feb85c1f28ff18f58891ffef132eef2fa09346aee33c28eb130ff28f5b766953334113211996d20011a198e3fc433f9f2541010ae17c1bf202580f6047472fb36857fe843b19f5984009ddc324044e847a4f4a0ab34f719595de37252d6235365e9b84392b061085349d73203a4a13e96f5432ec0fd4a1ee65accdd5e3904df54c1da510b0ff20dcc0c77fcb2c0e0eb605cb0504db87632cf3d8b4dae6e705769d1de354270123cb11450efc60ac47683d7b8d0f811365565fd98c4c8eb936bcab8d069fc33bd801b03adea2e1fbc5aa463d08ca19896d2bf59a071b851e6c239052172f296bfb5e72404790a2181014f3b94a4e97d117b438130368cc39dbb2d198065ae3986547926cd2162f40a29f0c3c8745c0f50fba3852e566d44575c29d39a03f0cda721984b6f440591f355e12d439ff150aab7613499dbd49adabc8676eef023b15b65bfc5ca06948109f23f350db82123535eb8a7433bdabcb909271a6ecbcb58b936a88cd4e8f2e6ff5800175f113253d8fa9ca8885c2f552e657dc603f252e1a8e308f76f0be79e2fb8f5d5fbbe2e30ecadd220723c8c0aea8078cdfcb3868263ff8f0940054da48781893a7e49ad5aff4af300cd804a6b6279ab3ff3afb64491c85194aab760d58a606654f9f4400e8b38591356fbf6425aca26dc85244259ff2b19c41b9f96f3ca9ec1dde434da7d2d392b905ddf3d1f9af93d1af5950bd493f5aa731b4056df31bd267b6b90a079831aaf579be0a39013137aac6d404f518cfd46840647e78bfe706ca4cf5e9c5453e9f7cfd2b8b4c8d169a44e55c88d4a9a7f9474241e221af44860018ab0856972e194cd934";

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex"))
            .collect()
    }

    fn hex_str(b: &[u8]) -> String {
        b.iter().map(|x| format!("{:02x}", x)).collect()
    }

    #[test]
    fn test_derive_initial_secret() {
        let dcid = hex("8394c8f03e515708");
        let initial_secret = hkdf_extract(&INITIAL_SALT, &dcid);
        assert_eq!(hex_str(&initial_secret), INITIAL_SECRET);
    }

    #[test]
    fn test_derive_client_keys() {
        let keys = derive_initial_keys(&hex("8394c8f03e515708"));
        assert_eq!(hex_str(&keys.key), CLIENT_INITIAL_KEY);
        assert_eq!(hex_str(&keys.iv), CLIENT_INITIAL_IV);
        assert_eq!(hex_str(&keys.hp), CLIENT_INITIAL_HP);
    }

    #[test]
    fn test_client_initial_secret() {
        let initial_secret = hkdf_extract(&INITIAL_SALT, &hex("8394c8f03e515708"));
        let client_secret = hkdf_expand_label(&initial_secret, CLIENT_IN_LABEL, b"", 32);
        assert_eq!(hex_str(&client_secret), CLIENT_INITIAL_SECRET);
    }

    #[test]
    fn test_varint() {
        assert_eq!(hex_str(&varint(0)), "00");
        assert_eq!(hex_str(&varint(1182)), "449e");
        assert_eq!(hex_str(&varint(37)), "25");
    }

    #[test]
    fn test_a2_full_packet() {
        let dcid = hex("8394c8f03e515708");
        let scid: Vec<u8> = Vec::new();
        let mut frames = hex(A2_PAYLOAD);
        frames.resize(1162, 0x00);
        let pkt = build_initial_packet(&dcid, &scid, 2, 4, &frames);
        assert_eq!(hex_str(&pkt), A2_PACKET);
        assert_eq!(pkt.len(), 1200);
    }

    #[test]
    #[ignore]
    fn live_probe_known_hosts() {
        use std::net::ToSocketAddrs;
        let targets = ["cloudflare.com", "www.google.com", "youtube.com", "discord.com"];
        for name in targets {
            let ips: Vec<std::net::IpAddr> = (name, 443)
                .to_socket_addrs()
                .map(|a| a.map(|x| x.ip()).collect())
                .unwrap_or_default();
            for ip in ips.into_iter().take(4) {
                let addr = SocketAddr::new(ip, 443);
                let ok = probe_quic(addr, name, 2, Duration::from_secs(2));
                println!("{} ({}) -> {}", name, ip, if ok { "REPLY" } else { "no reply" });
            }
        }
    }
}
