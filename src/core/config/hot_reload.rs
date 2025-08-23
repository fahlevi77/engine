//! Hot Reload Configuration Management
//! 
//! Provides hot reload capabilities for Siddhi configuration, allowing
//! runtime configuration changes without service restart.

use crate::core::config::{ConfigError, ConfigResult, SiddhiConfig};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::{broadcast, watch};
use tokio::time::interval;
use serde::{Deserialize, Serialize};

/// Configuration change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    /// Timestamp of the change
    pub timestamp: SystemTime,
    
    /// Type of configuration change
    pub change_type: ConfigChangeType,
    
    /// Path to the configuration that changed
    pub config_path: String,
    
    /// Previous configuration hash (if available)
    pub previous_hash: Option<String>,
    
    /// New configuration hash
    pub new_hash: String,
    
    /// Validation status of the new configuration
    pub validation_status: ValidationStatus,
    
    /// Additional metadata about the change
    pub metadata: HashMap<String, String>,
}

/// Types of configuration changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigChangeType {
    /// Configuration file was modified
    FileModified,
    
    /// Configuration file was created
    FileCreated,
    
    /// Configuration file was deleted
    FileDeleted,
    
    /// Environment variable changed
    EnvironmentChanged,
    
    /// Kubernetes ConfigMap updated
    ConfigMapUpdated,
    
    /// Kubernetes Secret updated
    SecretUpdated,
    
    /// API-driven configuration change
    ApiUpdate,
    
    /// External configuration source change
    ExternalSourceChanged,
}

/// Validation status for configuration changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    /// Configuration passed validation
    Valid,
    
    /// Configuration failed validation
    Invalid { errors: Vec<String> },
    
    /// Configuration validation is pending
    Pending,
    
    /// Configuration validation was skipped
    Skipped,
}

/// Hot reload configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadConfig {
    /// Enable hot reload functionality
    pub enabled: bool,
    
    /// Interval for checking configuration changes
    pub check_interval: Duration,
    
    /// Debounce duration to avoid rapid reloads
    pub debounce_duration: Duration,
    
    /// Maximum number of reload attempts
    pub max_retry_attempts: u32,
    
    /// Delay between retry attempts
    pub retry_delay: Duration,
    
    /// Validate configuration before applying
    pub validate_before_reload: bool,
    
    /// Backup configuration before applying changes
    pub backup_enabled: bool,
    
    /// Maximum number of configuration backups to keep
    pub max_backups: u32,
    
    /// Paths to watch for changes
    pub watch_paths: Vec<PathBuf>,
    
    /// File patterns to include in watching
    pub include_patterns: Vec<String>,
    
    /// File patterns to exclude from watching
    pub exclude_patterns: Vec<String>,
    
    /// Kubernetes ConfigMap watching configuration
    pub configmap_watch: Option<ConfigMapWatchConfig>,
    
    /// Kubernetes Secret watching configuration
    pub secret_watch: Option<SecretWatchConfig>,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for safety
            check_interval: Duration::from_secs(5),
            debounce_duration: Duration::from_millis(500),
            max_retry_attempts: 3,
            retry_delay: Duration::from_secs(1),
            validate_before_reload: true,
            backup_enabled: true,
            max_backups: 10,
            watch_paths: vec![PathBuf::from("config")],
            include_patterns: vec!["*.yaml".to_string(), "*.yml".to_string()],
            exclude_patterns: vec!["*.bak".to_string(), "*.tmp".to_string()],
            configmap_watch: None,
            secret_watch: None,
        }
    }
}

impl HotReloadConfig {
    /// Create development hot reload configuration
    pub fn development() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(1), // Faster checks in dev
            debounce_duration: Duration::from_millis(200),
            max_retry_attempts: 1, // Fail fast in dev
            retry_delay: Duration::from_millis(500),
            validate_before_reload: true,
            backup_enabled: false, // Don't clutter dev environment
            max_backups: 3,
            watch_paths: vec![PathBuf::from("config"), PathBuf::from(".")],
            include_patterns: vec!["*.yaml".to_string(), "*.yml".to_string(), "siddhi*.toml".to_string()],
            exclude_patterns: vec!["*.bak".to_string(), "*.tmp".to_string(), "target/**".to_string()],
            configmap_watch: None,
            secret_watch: None,
        }
    }
    
    /// Create production hot reload configuration
    pub fn production() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(30), // Less frequent in prod
            debounce_duration: Duration::from_secs(5), // Longer debounce
            max_retry_attempts: 5, // More resilient
            retry_delay: Duration::from_secs(2),
            validate_before_reload: true,
            backup_enabled: true,
            max_backups: 20, // More backups in prod
            watch_paths: vec![PathBuf::from("/etc/siddhi/config")],
            include_patterns: vec!["*.yaml".to_string(), "*.yml".to_string()],
            exclude_patterns: vec!["*.bak".to_string(), "*.tmp".to_string(), "*.lock".to_string()],
            configmap_watch: None,
            secret_watch: None,
        }
    }
    
    /// Create Kubernetes hot reload configuration
    pub fn kubernetes() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(10),
            debounce_duration: Duration::from_secs(2),
            max_retry_attempts: 3,
            retry_delay: Duration::from_secs(1),
            validate_before_reload: true,
            backup_enabled: true,
            max_backups: 5,
            watch_paths: vec![
                PathBuf::from("/etc/siddhi/config"),
                PathBuf::from("/var/secrets/siddhi"),
            ],
            include_patterns: vec!["*.yaml".to_string(), "*.yml".to_string()],
            exclude_patterns: vec!["*.bak".to_string(), "*.tmp".to_string()],
            configmap_watch: Some(ConfigMapWatchConfig::default()),
            secret_watch: Some(SecretWatchConfig::default()),
        }
    }
}

/// Kubernetes ConfigMap watching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapWatchConfig {
    /// ConfigMap name to watch
    pub configmap_name: String,
    
    /// Kubernetes namespace
    pub namespace: Option<String>,
    
    /// Label selector for ConfigMap watching
    pub label_selector: Option<String>,
    
    /// Fields to watch within the ConfigMap
    pub watch_fields: Vec<String>,
}

impl Default for ConfigMapWatchConfig {
    fn default() -> Self {
        Self {
            configmap_name: "siddhi-config".to_string(),
            namespace: None, // Use default namespace
            label_selector: Some("app=siddhi".to_string()),
            watch_fields: vec!["siddhi.yaml".to_string()],
        }
    }
}

/// Kubernetes Secret watching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretWatchConfig {
    /// Secret name to watch
    pub secret_name: String,
    
    /// Kubernetes namespace
    pub namespace: Option<String>,
    
    /// Label selector for Secret watching
    pub label_selector: Option<String>,
    
    /// Fields to watch within the Secret
    pub watch_fields: Vec<String>,
}

impl Default for SecretWatchConfig {
    fn default() -> Self {
        Self {
            secret_name: "siddhi-secrets".to_string(),
            namespace: None, // Use default namespace
            label_selector: Some("app=siddhi".to_string()),
            watch_fields: vec!["database-password".to_string(), "api-key".to_string()],
        }
    }
}

/// Hot reload manager handles configuration watching and reloading
pub struct HotReloadManager {
    /// Hot reload configuration
    config: HotReloadConfig,
    
    /// Current configuration state
    current_config: Arc<RwLock<SiddhiConfig>>,
    
    /// Configuration change event broadcaster
    change_broadcaster: broadcast::Sender<ConfigChangeEvent>,
    
    /// Configuration update channel
    config_update_sender: watch::Sender<SiddhiConfig>,
    
    /// File watchers for configuration files
    #[cfg(feature = "hot-reload")]
    file_watchers: HashMap<PathBuf, FileWatcher>,
    
    /// Kubernetes watchers
    #[cfg(all(feature = "hot-reload", feature = "kubernetes"))]
    k8s_watchers: Vec<KubernetesWatcher>,
    
    /// Configuration backup manager
    backup_manager: Option<ConfigBackupManager>,
    
    /// Last configuration hash for change detection
    last_config_hash: Arc<RwLock<String>>,
    
    /// Reload statistics
    reload_stats: Arc<RwLock<ReloadStatistics>>,
}

/// Reload statistics tracking
#[derive(Debug, Clone, Default)]
pub struct ReloadStatistics {
    /// Total number of reload attempts
    pub total_attempts: u64,
    
    /// Number of successful reloads
    pub successful_reloads: u64,
    
    /// Number of failed reloads
    pub failed_reloads: u64,
    
    /// Number of validation failures
    pub validation_failures: u64,
    
    /// Last reload timestamp
    pub last_reload_time: Option<SystemTime>,
    
    /// Last reload duration
    pub last_reload_duration: Option<Duration>,
    
    /// Average reload duration
    pub average_reload_duration: Duration,
}

impl HotReloadManager {
    /// Create a new hot reload manager
    pub fn new(
        config: HotReloadConfig,
        initial_config: SiddhiConfig,
    ) -> ConfigResult<(Self, watch::Receiver<SiddhiConfig>, broadcast::Receiver<ConfigChangeEvent>)> {
        let (change_broadcaster, change_receiver) = broadcast::channel(1000);
        let (config_update_sender, config_update_receiver) = watch::channel(initial_config.clone());
        
        let initial_hash = Self::calculate_config_hash(&initial_config)?;
        
        let backup_manager = if config.backup_enabled {
            Some(ConfigBackupManager::new(config.max_backups)?)
        } else {
            None
        };
        
        let manager = Self {
            config,
            current_config: Arc::new(RwLock::new(initial_config)),
            change_broadcaster,
            config_update_sender,
            #[cfg(feature = "hot-reload")]
            file_watchers: HashMap::new(),
            #[cfg(all(feature = "hot-reload", feature = "kubernetes"))]
            k8s_watchers: Vec::new(),
            backup_manager,
            last_config_hash: Arc::new(RwLock::new(initial_hash)),
            reload_stats: Arc::new(RwLock::new(ReloadStatistics::default())),
        };
        
        Ok((manager, config_update_receiver, change_receiver))
    }
    
    /// Start the hot reload system
    pub async fn start(&mut self) -> ConfigResult<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // Start file watching
        self.start_file_watching().await?;
        
        // Start Kubernetes watching if enabled
        #[cfg(all(feature = "hot-reload", feature = "kubernetes"))]
        self.start_kubernetes_watching().await?;
        
        // Start the main reload loop
        self.start_reload_loop().await;
        
        Ok(())
    }
    
    /// Start file system watching
    #[cfg(feature = "hot-reload")]
    async fn start_file_watching(&mut self) -> ConfigResult<()> {
        use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
        use std::sync::mpsc;
        
        let (tx, rx) = mpsc::channel();
        let mut watcher = watcher(tx, self.config.debounce_duration)
            .map_err(|e| ConfigError::internal_error(format!("Failed to create file watcher: {}", e)))?;
        
        // Watch configured paths
        for path in &self.config.watch_paths {
            watcher.watch(path, RecursiveMode::Recursive)
                .map_err(|e| ConfigError::internal_error(format!("Failed to watch path {:?}: {}", path, e)))?;
        }
        
        // Store the watcher (this keeps it alive)
        let file_watcher = FileWatcher { watcher, receiver: rx };
        self.file_watchers.insert(PathBuf::from("main"), file_watcher);
        
        Ok(())
    }
    
    /// Start file system watching (no-op when feature disabled)
    #[cfg(not(feature = "hot-reload"))]
    async fn start_file_watching(&mut self) -> ConfigResult<()> {
        // Hot reload feature not enabled
        Ok(())
    }
    
    /// Start Kubernetes resource watching
    #[cfg(all(feature = "hot-reload", feature = "kubernetes"))]
    async fn start_kubernetes_watching(&mut self) -> ConfigResult<()> {
        use kube::{Api, Client};
        use k8s_openapi::api::core::v1::{ConfigMap, Secret};
        use futures::TryStreamExt;
        
        let client = Client::try_default().await
            .map_err(|e| ConfigError::internal_error(format!("Failed to create K8s client: {}", e)))?;
        
        // Watch ConfigMaps
        if let Some(ref cm_config) = self.config.configmap_watch {
            let namespace = cm_config.namespace.as_deref().unwrap_or("default");
            let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
            
            let watcher = KubernetesWatcher::new_configmap(configmaps, cm_config.clone());
            self.k8s_watchers.push(watcher);
        }
        
        // Watch Secrets
        if let Some(ref secret_config) = self.config.secret_watch {
            let namespace = secret_config.namespace.as_deref().unwrap_or("default");
            let secrets: Api<Secret> = Api::namespaced(client.clone(), namespace);
            
            let watcher = KubernetesWatcher::new_secret(secrets, secret_config.clone());
            self.k8s_watchers.push(watcher);
        }
        
        Ok(())
    }
    
    /// Start Kubernetes watching (no-op when features disabled)
    #[cfg(not(all(feature = "hot-reload", feature = "kubernetes")))]
    async fn start_kubernetes_watching(&mut self) -> ConfigResult<()> {
        // Kubernetes hot reload not enabled
        Ok(())
    }
    
    /// Start the main reload loop
    async fn start_reload_loop(&self) {
        let mut interval = interval(self.config.check_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_and_reload().await {
                eprintln!("Hot reload check failed: {}", e);
            }
        }
    }
    
    /// Check for configuration changes and reload if necessary
    async fn check_and_reload(&self) -> ConfigResult<()> {
        // Check for file changes
        #[cfg(feature = "hot-reload")]
        self.check_file_changes().await?;
        
        // Check for Kubernetes changes
        #[cfg(all(feature = "hot-reload", feature = "kubernetes"))]
        self.check_kubernetes_changes().await?;
        
        Ok(())
    }
    
    /// Check for file system changes
    #[cfg(feature = "hot-reload")]
    async fn check_file_changes(&self) -> ConfigResult<()> {
        use notify::DebouncedEvent;
        use std::sync::mpsc::TryRecvError;
        
        for (path, watcher) in &self.file_watchers {
            loop {
                match watcher.receiver.try_recv() {
                    Ok(event) => {
                        match event {
                            DebouncedEvent::Write(ref file_path) | DebouncedEvent::Create(ref file_path) => {
                                if self.should_process_file(file_path) {
                                    self.handle_file_change(file_path.clone(), ConfigChangeType::FileModified).await?;
                                }
                            }
                            DebouncedEvent::Remove(ref file_path) => {
                                if self.should_process_file(file_path) {
                                    self.handle_file_change(file_path.clone(), ConfigChangeType::FileDeleted).await?;
                                }
                            }
                            _ => {} // Ignore other events
                        }
                    }
                    Err(TryRecvError::Empty) => break, // No more events
                    Err(TryRecvError::Disconnected) => {
                        return Err(ConfigError::internal_error("File watcher disconnected".to_string()));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check for file system changes (no-op when feature disabled)
    #[cfg(not(feature = "hot-reload"))]
    async fn check_file_changes(&self) -> ConfigResult<()> {
        Ok(())
    }
    
    /// Check for Kubernetes resource changes
    #[cfg(all(feature = "hot-reload", feature = "kubernetes"))]
    async fn check_kubernetes_changes(&self) -> ConfigResult<()> {
        for watcher in &self.k8s_watchers {
            if let Some(change) = watcher.check_for_changes().await? {
                self.handle_kubernetes_change(change).await?;
            }
        }
        
        Ok(())
    }
    
    /// Check for Kubernetes changes (no-op when features disabled)
    #[cfg(not(all(feature = "hot-reload", feature = "kubernetes")))]
    async fn check_kubernetes_changes(&self) -> ConfigResult<()> {
        Ok(())
    }
    
    /// Handle a file change event
    async fn handle_file_change(&self, file_path: PathBuf, change_type: ConfigChangeType) -> ConfigResult<()> {
        let start_time = SystemTime::now();
        
        // Increment attempt counter
        {
            let mut stats = self.reload_stats.write().unwrap();
            stats.total_attempts += 1;
        }
        
        // Create backup if enabled
        if let Some(ref backup_manager) = self.backup_manager {
            let current_config = self.current_config.read().unwrap().clone();
            backup_manager.create_backup(&current_config)?;
        }
        
        // Load new configuration
        let new_config = self.load_config_from_file(&file_path).await?;
        
        // Calculate hash for change detection
        let new_hash = Self::calculate_config_hash(&new_config)?;
        let previous_hash = {
            let hash = self.last_config_hash.read().unwrap().clone();
            if hash != new_hash {
                Some(hash)
            } else {
                return Ok(()); // No actual change
            }
        };
        
        // Validate configuration if enabled
        let validation_status = if self.config.validate_before_reload {
            match self.validate_configuration(&new_config).await {
                Ok(_) => ValidationStatus::Valid,
                Err(e) => {
                    let errors = vec![e.to_string()];
                    
                    // Increment validation failure counter
                    {
                        let mut stats = self.reload_stats.write().unwrap();
                        stats.validation_failures += 1;
                        stats.failed_reloads += 1;
                    }
                    
                    let event = ConfigChangeEvent {
                        timestamp: SystemTime::now(),
                        change_type,
                        config_path: file_path.to_string_lossy().to_string(),
                        previous_hash,
                        new_hash: new_hash.clone(),
                        validation_status: ValidationStatus::Invalid { errors: errors.clone() },
                        metadata: HashMap::new(),
                    };
                    
                    let _ = self.change_broadcaster.send(event);
                    
                    return Err(ConfigError::InternalError {
                        message: format!("Configuration validation failed: {}", errors.join(", "))
                    });
                }
            }
        } else {
            ValidationStatus::Skipped
        };
        
        // Apply configuration with retry logic
        let mut retry_count = 0;
        let mut last_error = None;
        
        while retry_count <= self.config.max_retry_attempts {
            match self.apply_configuration(new_config.clone()).await {
                Ok(_) => {
                    // Update hash
                    {
                        let mut hash = self.last_config_hash.write().unwrap();
                        *hash = new_hash.clone();
                    }
                    
                    // Update statistics
                    let reload_duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));
                    {
                        let mut stats = self.reload_stats.write().unwrap();
                        stats.successful_reloads += 1;
                        stats.last_reload_time = Some(SystemTime::now());
                        stats.last_reload_duration = Some(reload_duration);
                        
                        // Update average duration
                        if stats.successful_reloads == 1 {
                            stats.average_reload_duration = reload_duration;
                        } else {
                            let total_duration = stats.average_reload_duration * (stats.successful_reloads - 1) as u32 + reload_duration;
                            stats.average_reload_duration = total_duration / stats.successful_reloads as u32;
                        }
                    }
                    
                    // Broadcast success event
                    let event = ConfigChangeEvent {
                        timestamp: SystemTime::now(),
                        change_type,
                        config_path: file_path.to_string_lossy().to_string(),
                        previous_hash,
                        new_hash,
                        validation_status,
                        metadata: {
                            let mut metadata = HashMap::new();
                            metadata.insert("reload_duration_ms".to_string(), reload_duration.as_millis().to_string());
                            metadata.insert("retry_count".to_string(), retry_count.to_string());
                            metadata
                        },
                    };
                    
                    let _ = self.change_broadcaster.send(event);
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                    retry_count += 1;
                    
                    if retry_count <= self.config.max_retry_attempts {
                        tokio::time::sleep(self.config.retry_delay).await;
                    }
                }
            }
        }
        
        // All retries failed
        {
            let mut stats = self.reload_stats.write().unwrap();
            stats.failed_reloads += 1;
        }
        
        let error = last_error.unwrap_or_else(|| ConfigError::internal_error("Unknown error".to_string()));
        
        let event = ConfigChangeEvent {
            timestamp: SystemTime::now(),
            change_type,
            config_path: file_path.to_string_lossy().to_string(),
            previous_hash,
            new_hash,
            validation_status: ValidationStatus::Invalid { errors: vec![error.to_string()] },
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("retry_count".to_string(), retry_count.to_string());
                metadata.insert("max_retries_exceeded".to_string(), "true".to_string());
                metadata
            },
        };
        
        let _ = self.change_broadcaster.send(event);
        
        Err(error)
    }
    
    /// Handle Kubernetes resource changes
    #[cfg(all(feature = "hot-reload", feature = "kubernetes"))]
    async fn handle_kubernetes_change(&self, change: KubernetesChange) -> ConfigResult<()> {
        // Similar to handle_file_change but for Kubernetes resources
        // Implementation would be similar but adapted for K8s resources
        Ok(())
    }
    
    /// Determine if a file should be processed based on include/exclude patterns
    fn should_process_file(&self, file_path: &Path) -> bool {
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Check exclude patterns first
        for pattern in &self.config.exclude_patterns {
            if self.matches_pattern(file_name, pattern) {
                return false;
            }
        }
        
        // Check include patterns
        for pattern in &self.config.include_patterns {
            if self.matches_pattern(file_name, pattern) {
                return true;
            }
        }
        
        false
    }
    
    /// Simple pattern matching (supports * wildcard)
    fn matches_pattern(&self, text: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // Simple glob matching
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                text.starts_with(parts[0]) && text.ends_with(parts[1])
            } else {
                // More complex patterns could be supported with regex
                text == pattern
            }
        } else {
            text == pattern
        }
    }
    
    /// Load configuration from a file
    async fn load_config_from_file(&self, file_path: &Path) -> ConfigResult<SiddhiConfig> {
        use crate::core::config::ConfigManager;
        
        let manager = ConfigManager::new();
        manager.load_from_file(file_path).await
    }
    
    /// Validate configuration before applying
    async fn validate_configuration(&self, config: &SiddhiConfig) -> ConfigResult<()> {
        use crate::core::config::validator::ConfigValidator;
        
        let validator = ConfigValidator::new();
        let result = validator.validate(config).await;
        
        if result.is_valid() {
            Ok(())
        } else {
            Err(ConfigError::validation_error(result.errors))
        }
    }
    
    /// Apply new configuration
    async fn apply_configuration(&self, new_config: SiddhiConfig) -> ConfigResult<()> {
        // Update current configuration
        {
            let mut current = self.current_config.write().unwrap();
            *current = new_config.clone();
        }
        
        // Send update through watch channel
        self.config_update_sender.send(new_config)
            .map_err(|_| ConfigError::internal_error("Failed to send configuration update".to_string()))?;
        
        Ok(())
    }
    
    /// Calculate hash for configuration change detection
    pub fn calculate_config_hash(config: &SiddhiConfig) -> ConfigResult<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Serialize config to bytes for hashing
        let serialized = serde_json::to_vec(config)
            .map_err(|e| ConfigError::serialization_error(format!("Failed to serialize config for hashing: {}", e)))?;
        
        let mut hasher = DefaultHasher::new();
        serialized.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }
    
    /// Get reload statistics
    pub fn get_reload_statistics(&self) -> ReloadStatistics {
        self.reload_stats.read().unwrap().clone()
    }
    
    /// Force reload configuration
    pub async fn force_reload(&self) -> ConfigResult<()> {
        // Reload from all configured sources
        for path in &self.config.watch_paths {
            if path.exists() {
                for file in std::fs::read_dir(path)
                    .map_err(|e| ConfigError::InternalError { message: format!("Failed to read directory {:?}: {}", path, e) })? 
                {
                    let file = file
                        .map_err(|e| ConfigError::InternalError { message: format!("Failed to read directory entry: {}", e) })?;
                    let file_path = file.path();
                    
                    if self.should_process_file(&file_path) {
                        self.handle_file_change(file_path, ConfigChangeType::ApiUpdate).await?;
                        break; // Only reload first matching file to avoid conflicts
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// File watcher wrapper
#[cfg(feature = "hot-reload")]
struct FileWatcher {
    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,
    receiver: std::sync::mpsc::Receiver<notify::DebouncedEvent>,
}

/// Kubernetes resource watcher
#[cfg(all(feature = "hot-reload", feature = "kubernetes"))]
struct KubernetesWatcher {
    // Implementation would depend on specific Kubernetes watching requirements
}

#[cfg(all(feature = "hot-reload", feature = "kubernetes"))]
impl KubernetesWatcher {
    fn new_configmap(_configmaps: kube::Api<k8s_openapi::api::core::v1::ConfigMap>, _config: ConfigMapWatchConfig) -> Self {
        Self {}
    }
    
    fn new_secret(_secrets: kube::Api<k8s_openapi::api::core::v1::Secret>, _config: SecretWatchConfig) -> Self {
        Self {}
    }
    
    async fn check_for_changes(&self) -> ConfigResult<Option<KubernetesChange>> {
        // Implementation would check for actual K8s resource changes
        Ok(None)
    }
}

/// Kubernetes change event
#[cfg(all(feature = "hot-reload", feature = "kubernetes"))]
struct KubernetesChange {
    // Fields would describe the specific change
}

/// Configuration backup manager
pub struct ConfigBackupManager {
    max_backups: u32,
    backup_dir: PathBuf,
}

impl ConfigBackupManager {
    pub fn new(max_backups: u32) -> ConfigResult<Self> {
        let backup_dir = PathBuf::from("config/backups");
        std::fs::create_dir_all(&backup_dir)
            .map_err(|e| ConfigError::InternalError { message: format!("Failed to create backup directory: {}", e) })?;
        
        Ok(Self {
            max_backups,
            backup_dir,
        })
    }
    
    pub fn create_backup(&self, config: &SiddhiConfig) -> ConfigResult<PathBuf> {
        // Ensure backup directory exists
        std::fs::create_dir_all(&self.backup_dir)
            .map_err(|e| ConfigError::InternalError { message: format!("Failed to create backup directory: {}", e) })?;
            
        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        let backup_file = self.backup_dir.join(format!("siddhi-config-backup-{}.yaml", timestamp));
        
        // Serialize and save configuration
        let yaml_content = serde_yaml::to_string(config)
            .map_err(|e| ConfigError::serialization_error(format!("Failed to serialize config for backup: {}", e)))?;
        
        std::fs::write(&backup_file, yaml_content)
            .map_err(|e| ConfigError::InternalError { message: format!("Failed to write backup file: {}", e) })?;
        
        // Clean up old backups
        self.cleanup_old_backups()?;
        
        Ok(backup_file)
    }
    
    fn cleanup_old_backups(&self) -> ConfigResult<()> {
        let mut backup_files: Vec<_> = std::fs::read_dir(&self.backup_dir)
            .map_err(|e| ConfigError::InternalError { message: format!("Failed to read backup directory: {}", e) })?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name().to_string_lossy().starts_with("siddhi-config-backup-")
            })
            .collect();
        
        // Sort by modification time (newest first)
        backup_files.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
            let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
            b_time.cmp(&a_time)
        });
        
        // Remove excess backups
        for file_entry in backup_files.iter().skip(self.max_backups as usize) {
            if let Err(e) = std::fs::remove_file(file_entry.path()) {
                eprintln!("Failed to remove old backup file {:?}: {}", file_entry.path(), e);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    
    #[tokio::test]
    async fn test_hot_reload_config_creation() {
        let config = HotReloadConfig::default();
        assert!(!config.enabled); // Disabled by default
        assert_eq!(config.check_interval, Duration::from_secs(5));
        assert_eq!(config.max_retry_attempts, 3);
        assert!(config.validate_before_reload);
        assert!(config.backup_enabled);
    }
    
    #[tokio::test]
    async fn test_hot_reload_development_config() {
        let config = HotReloadConfig::development();
        assert!(config.enabled);
        assert_eq!(config.check_interval, Duration::from_secs(1));
        assert_eq!(config.max_retry_attempts, 1);
        assert!(!config.backup_enabled); // Disabled in dev
        assert_eq!(config.max_backups, 3);
    }
    
    #[tokio::test]
    async fn test_hot_reload_production_config() {
        let config = HotReloadConfig::production();
        assert!(config.enabled);
        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.max_retry_attempts, 5);
        assert!(config.backup_enabled);
        assert_eq!(config.max_backups, 20);
    }
    
    #[tokio::test]
    async fn test_hot_reload_kubernetes_config() {
        let config = HotReloadConfig::kubernetes();
        assert!(config.enabled);
        assert_eq!(config.check_interval, Duration::from_secs(10));
        assert!(config.configmap_watch.is_some());
        assert!(config.secret_watch.is_some());
        
        let cm_watch = config.configmap_watch.unwrap();
        assert_eq!(cm_watch.configmap_name, "siddhi-config");
        assert_eq!(cm_watch.label_selector, Some("app=siddhi".to_string()));
    }
    
    #[tokio::test]
    async fn test_config_change_event() {
        let event = ConfigChangeEvent {
            timestamp: SystemTime::now(),
            change_type: ConfigChangeType::FileModified,
            config_path: "/etc/siddhi/config.yaml".to_string(),
            previous_hash: Some("old_hash".to_string()),
            new_hash: "new_hash".to_string(),
            validation_status: ValidationStatus::Valid,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("reload_duration_ms".to_string(), "150".to_string());
                metadata
            },
        };
        
        assert!(matches!(event.change_type, ConfigChangeType::FileModified));
        assert!(matches!(event.validation_status, ValidationStatus::Valid));
        assert_eq!(event.config_path, "/etc/siddhi/config.yaml");
        assert_eq!(event.metadata.get("reload_duration_ms"), Some(&"150".to_string()));
    }
    
    #[test]
    fn test_pattern_matching() {
        let config = HotReloadConfig::default();
        let manager_config = HotReloadConfig::default();
        let manager = HotReloadManager {
            config: manager_config,
            current_config: Arc::new(RwLock::new(SiddhiConfig::default())),
            change_broadcaster: broadcast::channel(1).0,
            config_update_sender: watch::channel(SiddhiConfig::default()).0,
            #[cfg(feature = "hot-reload")]
            file_watchers: HashMap::new(),
            #[cfg(all(feature = "hot-reload", feature = "kubernetes"))]
            k8s_watchers: Vec::new(),
            backup_manager: None,
            last_config_hash: Arc::new(RwLock::new("test_hash".to_string())),
            reload_stats: Arc::new(RwLock::new(ReloadStatistics::default())),
        };
        
        // Test wildcard matching
        assert!(manager.matches_pattern("config.yaml", "*.yaml"));
        assert!(manager.matches_pattern("siddhi.yml", "*.yml"));
        assert!(!manager.matches_pattern("config.txt", "*.yaml"));
        
        // Test exact matching
        assert!(manager.matches_pattern("config.yaml", "config.yaml"));
        assert!(!manager.matches_pattern("config.yml", "config.yaml"));
    }
    
    #[tokio::test]
    async fn test_config_backup_manager() {
        let temp_dir = tempdir().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        
        let backup_manager = ConfigBackupManager {
            max_backups: 3,
            backup_dir: backup_dir.clone(),
        };
        
        let config = SiddhiConfig::default();
        
        // Create first backup
        let backup_path = backup_manager.create_backup(&config).unwrap();
        assert!(backup_path.exists());
        assert!(backup_path.file_name().unwrap().to_string_lossy().starts_with("siddhi-config-backup-"));
        
        // Verify backup content
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        let restored_config: SiddhiConfig = serde_yaml::from_str(&backup_content).unwrap();
        assert_eq!(restored_config.siddhi.runtime.mode, config.siddhi.runtime.mode);
    }
    
    #[tokio::test]
    async fn test_reload_statistics() {
        let mut stats = ReloadStatistics::default();
        
        assert_eq!(stats.total_attempts, 0);
        assert_eq!(stats.successful_reloads, 0);
        assert_eq!(stats.failed_reloads, 0);
        assert!(stats.last_reload_time.is_none());
        
        // Simulate some statistics
        stats.total_attempts = 5;
        stats.successful_reloads = 4;
        stats.failed_reloads = 1;
        stats.last_reload_time = Some(SystemTime::now());
        stats.last_reload_duration = Some(Duration::from_millis(250));
        stats.average_reload_duration = Duration::from_millis(200);
        
        assert_eq!(stats.total_attempts, 5);
        assert_eq!(stats.successful_reloads, 4);
        assert_eq!(stats.failed_reloads, 1);
        assert!(stats.last_reload_time.is_some());
    }
    
    #[tokio::test]
    async fn test_config_hash_calculation() {
        let config1 = SiddhiConfig::default();
        let mut config2 = SiddhiConfig::default();
        config2.siddhi.runtime.performance.thread_pool_size = 16;
        
        let hash1 = HotReloadManager::calculate_config_hash(&config1).unwrap();
        let hash2 = HotReloadManager::calculate_config_hash(&config2).unwrap();
        
        assert_ne!(hash1, hash2); // Different configs should have different hashes
        
        let hash1_repeat = HotReloadManager::calculate_config_hash(&config1).unwrap();
        assert_eq!(hash1, hash1_repeat); // Same config should produce same hash
    }
}