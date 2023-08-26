
pub struct Pattern {
    pub id: u32,
    pub pattern: &'static str,
    pub description: &'static str,
}

pub static PYTHON_SUB_PATTERNS: &[Pattern; 4] = &[
    Pattern{id: 1, pattern: r"^.*django_settings_module.*", description: "django settings path"},
    Pattern{id: 2, pattern: r"^.*django_secret_key.*$", description: "django secret key"},
    Pattern{id: 3, pattern: r"^.*password.*$", description: "password found"},
    Pattern{id: 4, pattern: r"^.*secret.*$", description: "secret found"},
];