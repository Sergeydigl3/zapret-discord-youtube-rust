#![cfg(target_os = "windows")]

use std::process::Command;

pub fn add_defender_exclusion() -> Result<(), String> {
    let cache_dir = crate::config::get_cache_dir();
    let path_str = cache_dir.to_str().unwrap();
    
    let status = Command::new("powershell")
        .arg("-Command")
        .arg(format!("Start-Process powershell -ArgumentList '-WindowStyle Hidden -Command Add-MpPreference -ExclusionPath \"{}\"' -Verb RunAs -Wait", path_str))
        .status()
        .map_err(|e| format!("Failed to execute powershell: {}", e))?;
        
    if status.success() {
        Ok(())
    } else {
        Err("Failed to add exclusion. UAC prompt might have been denied.".to_string())
    }
}

pub fn remove_defender_exclusion() -> Result<(), String> {
    let cache_dir = crate::config::get_cache_dir();
    let path_str = cache_dir.to_str().unwrap();
    
    let status = Command::new("powershell")
        .arg("-Command")
        .arg(format!("Start-Process powershell -ArgumentList '-WindowStyle Hidden -Command Remove-MpPreference -ExclusionPath \"{}\"' -Verb RunAs -Wait", path_str))
        .status()
        .map_err(|e| format!("Failed to execute powershell: {}", e))?;
        
    if status.success() {
        Ok(())
    } else {
        Err("Failed to remove exclusion. UAC prompt might have been denied.".to_string())
    }
}

pub fn check_defender_exclusion() -> Result<bool, String> {
    let cache_dir = crate::config::get_cache_dir();
    let path_str = cache_dir.to_str().unwrap().to_lowercase();
    
    let output = Command::new("powershell")
        .arg("-Command")
        .arg("Get-MpPreference | Select-Object -ExpandProperty ExclusionPath")
        .output()
        .map_err(|e| format!("Failed to execute powershell: {}", e))?;
        
    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
    
    if !output.status.success() || stdout.contains("administrator") {
        return Err("Needs Admin/Failed".to_string());
    }
    
    Ok(stdout.contains(&path_str))
}
