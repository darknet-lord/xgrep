
pub struct Pattern {
    pub name: &'static str,
    pub pattern: &'static str,
    pub description: &'static str,
}

pub static PYTHON_SUB_PATTERNS: &[Pattern; 4] = &[
    Pattern{name: "django_settings", pattern: r"^.*django_settings_module.*", description: "django settings path"},
    Pattern{name: "django_secret", pattern: "^.*django_secret_key.*$", description: "django secret key"},
    Pattern{name: "password", pattern: "^.*password.*$", description: "password found"},
    Pattern{name: "secret", pattern: "^.*secret.*$", description: "secret found"},
];