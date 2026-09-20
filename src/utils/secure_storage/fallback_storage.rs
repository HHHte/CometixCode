//! Maps to: CC `utils/secureStorage/fallbackStorage.ts`.

use super::{SecureStorageBackend, SecureStorageData, SecureStorageUpdate};

pub(crate) struct FallbackStorage {
    name: String,
    primary: Box<dyn SecureStorageBackend>,
    secondary: Box<dyn SecureStorageBackend>,
}

impl SecureStorageBackend for FallbackStorage {
    /// Maps to: CC `utils/secureStorage/fallbackStorage.ts:12` returned `name`.
    fn name(&self) -> &str {
        &self.name
    }

    /// Maps to: CC `utils/secureStorage/fallbackStorage.ts:13-19` returned `read`.
    fn read(&self) -> Option<SecureStorageData> {
        self.primary
            .read()
            .or_else(|| self.secondary.read())
            .or_else(|| Some(serde_json::json!({})))
    }

    /// Maps to: CC `utils/secureStorage/fallbackStorage.ts:27-61` returned `update`.
    /// L2 (`OAuth credential side-effect closure`): a typed closed-outlet error
    /// is propagated and never converted into a secondary credential write.
    fn update(&self, data: &SecureStorageData) -> anyhow::Result<SecureStorageUpdate> {
        let primary_data_before = self.primary.read();
        let primary_result = self.primary.update(data)?;
        if primary_result.success {
            if primary_data_before.is_none() {
                let _ = self.secondary.delete()?;
            }
            return Ok(primary_result);
        }

        let fallback_result = self.secondary.update(data)?;
        if fallback_result.success {
            if primary_data_before.is_some() {
                let _ = self.primary.delete()?;
            }
            return Ok(SecureStorageUpdate {
                success: true,
                warning: fallback_result.warning,
            });
        }

        Ok(SecureStorageUpdate {
            success: false,
            warning: None,
        })
    }

    /// Maps to: CC `utils/secureStorage/fallbackStorage.ts:63-68` returned `delete`.
    fn delete(&self) -> anyhow::Result<bool> {
        let primary_success = self.primary.delete()?;
        let secondary_success = self.secondary.delete()?;
        Ok(primary_success || secondary_success)
    }
}

/// Maps to: CC `utils/secureStorage/fallbackStorage.ts:7-62`
/// `createFallbackStorage`.
pub(crate) fn create_fallback_storage(
    primary: Box<dyn SecureStorageBackend>,
    secondary: Box<dyn SecureStorageBackend>,
) -> FallbackStorage {
    let name = format!("{}-with-{}-fallback", primary.name(), secondary.name());
    FallbackStorage {
        name,
        primary,
        secondary,
    }
}
