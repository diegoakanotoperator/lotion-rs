use hunspell_rs::Hunspell;
use std::sync::Mutex;
use tauri::State;

pub struct SafeHunspell(pub Hunspell);
unsafe impl Send for SafeHunspell {}
unsafe impl Sync for SafeHunspell {}

pub struct SpellcheckManager {
    pub hunspell: Mutex<Option<SafeHunspell>>,
}

impl SpellcheckManager {
    pub fn new() -> Self {
        let manager = Self {
            hunspell: Mutex::new(None),
        };

        // Try to load default en_US dictionary on Linux
        let aff = "/usr/share/hunspell/en_US.aff";
        let dic = "/usr/share/hunspell/en_US.dic";

        if std::path::Path::new(aff).exists() && std::path::Path::new(dic).exists() {
            manager.load_dictionaries(aff, dic);
        } else {
            log::warn!("SpellcheckManager: Default en_US dictionaries not found in /usr/share/hunspell/");
        }

        manager
    }
    
    pub fn load_dictionaries(&self, aff_path: &str, dic_path: &str) {
        let hs = Hunspell::new(aff_path, dic_path);
        *self.hunspell.lock().unwrap() = Some(SafeHunspell(hs));
        log::info!("Hunspell dictionaries loaded successfully.");
    }
}

#[tauri::command]
pub fn check_spelling(word: String, state: State<'_, SpellcheckManager>) -> bool {
    let hs_lock = state.hunspell.lock().unwrap();
    if let Some(hs) = hs_lock.as_ref() {
        hs.0.check(&word)
    } else {
        true 
    }
}

#[tauri::command]
pub fn get_spelling_suggestions(word: String, state: State<'_, SpellcheckManager>) -> Vec<String> {
    let hs_lock = state.hunspell.lock().unwrap();
    if let Some(hs) = hs_lock.as_ref() {
        hs.0.suggest(&word)
    } else {
        Vec::new()
    }
}
