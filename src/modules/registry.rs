use actix_web::web;

use crate::cron::CronRegistry;

/// Module trait for registering routes, jobs, permissions, and OpenAPI specs
pub trait AppModule: Send + Sync {
    fn name(&self) -> &'static str;
    fn register_routes(&self, cfg: &mut web::ServiceConfig);
    fn register_jobs(&self, registry: &mut CronRegistry);
    fn register_permissions(&self, registry: &mut PermissionRegistry);
    fn register_openapi(&self, registry: &mut OpenApiRegistry);
}

/// Registry for managing application modules
pub struct ModuleRegistry {
    pub modules: Vec<Box<dyn AppModule>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn register(&mut self, module: Box<dyn AppModule>) {
        tracing::info!("Registering module: {}", module.name());
        self.modules.push(module);
    }

    pub fn get_module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn get_module_names(&self) -> Vec<&str> {
        self.modules.iter().map(|m| m.name()).collect()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission registry for role-based access control
pub struct PermissionRegistry {
    permissions: Vec<Permission>,
}

impl PermissionRegistry {
    pub fn new() -> Self {
        Self {
            permissions: Vec::new(),
        }
    }

    pub fn register_permission(&mut self, permission: Permission) {
        self.permissions.push(permission);
    }

    pub fn get_permissions(&self) -> &[Permission] {
        &self.permissions
    }
}

impl Default for PermissionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission definition
#[derive(Debug, Clone)]
pub struct Permission {
    pub name: String,
    pub description: String,
    pub resource: String,
    pub action: String,
}

/// OpenAPI registry for documentation
pub struct OpenApiRegistry {
    schemas: Vec<OpenApiSchema>,
}

impl OpenApiRegistry {
    pub fn new() -> Self {
        Self {
            schemas: Vec::new(),
        }
    }

    pub fn register_schema(&mut self, schema: OpenApiSchema) {
        self.schemas.push(schema);
    }

    pub fn get_schemas(&self) -> &[OpenApiSchema] {
        &self.schemas
    }
}

impl Default for OpenApiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// OpenAPI schema definition
#[derive(Debug, Clone)]
pub struct OpenApiSchema {
    pub name: String,
    pub definition: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestModule;

    impl AppModule for TestModule {
        fn name(&self) -> &'static str {
            "test"
        }

        fn register_routes(&self, _cfg: &mut web::ServiceConfig) {}
        fn register_jobs(&self, _registry: &mut CronRegistry) {}
        fn register_permissions(&self, _registry: &mut PermissionRegistry) {}
        fn register_openapi(&self, _registry: &mut OpenApiRegistry) {}
    }

    #[test]
    fn test_module_registry() {
        let mut registry = ModuleRegistry::new();
        assert_eq!(registry.get_module_count(), 0);

        registry.register(Box::new(TestModule));
        assert_eq!(registry.get_module_count(), 1);
        assert_eq!(registry.get_module_names(), vec!["test"]);
    }

    #[test]
    fn test_permission_registry() {
        let mut registry = PermissionRegistry::new();
        
        let permission = Permission {
            name: "users:read".to_string(),
            description: "Read user data".to_string(),
            resource: "users".to_string(),
            action: "read".to_string(),
        };

        registry.register_permission(permission.clone());
        assert_eq!(registry.get_permissions().len(), 1);
        assert_eq!(registry.get_permissions()[0].name, "users:read");
    }
}