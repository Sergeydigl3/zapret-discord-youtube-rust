# Methods
1. `is_installed(&self) -> bool`
2. `is_active(&self) -> bool`
3. `install(&self, exe_path: &Path, config_path: &Path, cache_dir: &Path) -> Result<(), String>`
4. `uninstall(&self) -> Result<(), String>
start(&self) -> Result<(), String>
stop(&self) -> Result<(), String>`
5. `restart(&self) -> Result<(), String>`