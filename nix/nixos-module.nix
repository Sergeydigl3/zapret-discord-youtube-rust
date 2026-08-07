{ config, lib, pkgs, utils, ... }:

let
  cfg = config.services.zapret-rust;
  inherit (lib)
    types mkOption mkEnableOption mkIf mkMerge literalExpression;

  cacheDir = lib.removeSuffix "/" (toString cfg.cacheDirectory);
  defaultStateDir = "/var/lib/zapret-rust";

  configFile = pkgs.writeText "zapret-rust.conf.env" ''
    interface=${cfg.interface}
    strategy=${cfg.strategy}
    gamefiltertcp=${if cfg.gamefilterTcp then "true" else "false"}
    gamefilterudp=${if cfg.gamefilterUdp then "true" else "false"}
    backend=${cfg.backend}
    ${cfg.extraConfig}
  '';

  strategiesSrc = pkgs.fetchzip {
    url = "https://github.com/Flowseal/zapret-discord-youtube/archive/${cfg.strategies.version}.zip";
    hash = cfg.strategies.hash;
    stripRoot = true;
  };

  provisionScript = pkgs.writeShellScript "zapret-rust-provision" ''
    set -eu
    cache='${cacheDir}'
    bin_dir="$cache/bin"
    strat_dir="$cache/zapret-discord-youtube-linux"

    mkdir -p "$bin_dir" "$strat_dir"

    if [ ! -f "$bin_dir/.zapret-rust-source" ] ||
      [ "$(cat "$bin_dir/.zapret-rust-source")" != '${cfg.nfqwsPackage}' ]; then
      cp '${cfg.nfqwsPackage}/bin/nfqws' "$bin_dir/nfqws"
      printf '%s\n' '${cfg.nfqwsPackage}' > "$bin_dir/.zapret-rust-source"
    fi

    if [ ! -f "$strat_dir/.zapret-rust-source" ] ||
      [ "$(cat "$strat_dir/.zapret-rust-source")" != '${strategiesSrc}' ]; then
      cp -r '${strategiesSrc}/.' "$strat_dir/"
      printf '%s\n' '${strategiesSrc}' > "$strat_dir/.zapret-rust-source"
    fi
  '';
in
{
  options.services.zapret-rust = {
    enable = mkEnableOption "Zapret Discord & YouTube DPI-bypass daemon (zapret-rust)";

    package = mkOption {
      type = types.nullOr types.package;
      default = null;
      example = literalExpression "inputs.zapret-rust.packages.${pkgs.system}.default";
      description = ''
        The `zapret-rust` package to use.

        This option is automatically populated when the module is imported
        from the `zapret-rust` flake via `nixosModules.zapret-rust`.
        Set manually only if you need to override the default package.
      '';
    };

    interface = mkOption {
      type = types.str;
      default = "any";
      description = ''
        Network interface to filter on. Use `any` to match all interfaces.
      '';
    };

    strategy = mkOption {
      type = types.str;
      default = "general.bat";
      description = ''
        Strategy file (.bat) to apply, relative to the strategy repository
        in the cache directory (e.g. `general.bat`, `general (ALT).bat`).
      '';
    };

    gamefilterTcp = mkEnableOption "the TCP game filter (ports 50000-50100)";
    gamefilterUdp = mkEnableOption "the UDP game filter (ports 50000-50100)";

    backend = mkOption {
      type = types.enum [ "nftables" "iptables" ];
      default = "nftables";
      description = ''
        Firewall backend used to redirect traffic into the nfqueue.
      '';
    };

    cacheDirectory = mkOption {
      type = types.path;
      default = defaultStateDir;
      description = ''
        Working directory for downloaded dependencies (nfqws binary, strategy
        repository), configuration and logs.

        When `provision` is enabled (default) the directory is populated
        automatically on first start.
      '';
    };

    provision = mkOption {
      type = types.bool;
      default = true;
      description = ''
        Automatically populate `cacheDirectory` with the nfqws binary (from
        `nfqwsPackage`) and the strategies repository.
      '';
    };

    nfqwsPackage = mkOption {
      type = types.package;
      default = pkgs.zapret;
      example = literalExpression "pkgs.zapret";
      description = ''
        Package providing the `nfqws` binary. Override only if you need
        a specific version or build.
      '';
    };

    strategies = mkOption {
      description = ''
        The `zapret-discord-youtube` strategies repository source: the commit
        reference and the matching archive hash. When changing `version`, set
        `hash` to the hash of the new archive (obtainable via
        `nix-prefetch-url <url>`).
      '';
      type = types.submodule {
        options = {
          version = mkOption {
            type = types.str;
            default = "9503dc045133000af8075e066f09bb469008e530";
            description = "Commit hash (or other archive reference) of the strategies repository.";
          };
          hash = mkOption {
            type = types.str;
            default = "sha256-Lf7oloMkMziQRzyE/nw16J1/hAxSuRG5ioZb7UbLRUo=";
            description = "SHA-256 (SRI format) of the repository archive.";
          };
        };
      };
    };

    extraConfig = mkOption {
      type = types.lines;
      default = "";
      description = ''
        Extra lines appended to the generated `conf.env` file.
      '';
    };

    serviceConfig = mkOption {
      type = lib.types.attrsOf utils.systemdUtils.unitOptions.unitOption;
      default = { };
      example = literalExpression ''
        {
          MemoryMax = "512M";
        }
      '';
      description = ''
        Additional systemd service options merged into the unit configuration.
        Values are merged attribute-wise, so overrides take precedence over the
        module's defaults.
      '';
    };
  };

  config = mkIf cfg.enable (mkMerge [
    {
      assertions = [
        {
          assertion = cfg.package != null;
          message = ''
            services.zapret-rust.package must be set when enable = true.
            This is normally handled automatically when importing the module
            from the zapret-rust flake.
          '';
        }
      ];
    }

    {
      systemd.services.zapret-rust = {
        description = "Zapret Discord Youtube Service";

        wants = [ "network-online.target" ];
        after = [ "network-online.target" ];

        path = with pkgs; [
          nftables
          iptables
          libcap
          procps
          curl
          coreutils
        ];

        serviceConfig = mkMerge [
          {
            Type = "simple";

            ExecStart = lib.concatStringsSep " " ([
              "${cfg.package}/bin/zapret-rust"
              "--config" (lib.escapeShellArg (toString configFile))
              "--cache-dir" (lib.escapeShellArg cacheDir)
            ]);

            Restart = "always";
            RestartSec = "5s";

            PrivateTmp = true;
            ProtectSystem = "strict";
            ProtectHome = true;

            StandardOutput = "journal";
            StandardError = "journal";
            SyslogIdentifier = "zapret-rust";
          }
          (mkIf (cacheDir == defaultStateDir) {
            StateDirectory = "zapret-rust";
          })
          (mkIf (cacheDir != defaultStateDir) {
            ReadWritePaths = [ cacheDir ];
          })
          (mkIf cfg.provision {
            ExecStartPre = [ (toString provisionScript) ];
          })
          cfg.serviceConfig
        ];
      };
    }
  ]);
}
