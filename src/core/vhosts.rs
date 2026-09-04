use std::path::Path;

/// Generate Apache vhost configuration.
pub fn generate_apache_vhost(
    project_name: &str,
    document_root: &Path,
    server_name: &str,
    port: u16,
    ssl_port: u16,
    ssl_enabled: bool,
) -> String {
    let root = document_root.to_string_lossy().replace('\\', "/");
    let mut vhost = format!(
        r#"# BoxedCat auto-generated vhost: {project_name}
<VirtualHost *:{port}>
    ServerName {server_name}
    DocumentRoot "{root}"

    <Directory "{root}">
        Options Indexes FollowSymLinks MultiViews
        AllowOverride All
        Require all granted
    </Directory>

    ErrorLog "logs/{project_name}-error.log"
    CustomLog "logs/{project_name}-access.log" common
</VirtualHost>
"#
    );

    if ssl_enabled {
        vhost += &format!(
            r#"
<VirtualHost *:{ssl_port}>
    ServerName {server_name}
    DocumentRoot "{root}"

    SSLEngine on
    SSLCertificateFile "conf/ssl/{project_name}.crt"
    SSLCertificateKeyFile "conf/ssl/{project_name}.key"

    <Directory "{root}">
        Options Indexes FollowSymLinks MultiViews
        AllowOverride All
        Require all granted
    </Directory>

    ErrorLog "logs/{project_name}-ssl-error.log"
    CustomLog "logs/{project_name}-ssl-access.log" common
</VirtualHost>
"#
        );
    }

    vhost
}

/// Generate Nginx vhost configuration.
pub fn generate_nginx_vhost(
    project_name: &str,
    document_root: &Path,
    server_name: &str,
    port: u16,
    ssl_port: u16,
    ssl_enabled: bool,
) -> String {
    let root = document_root.to_string_lossy().replace('\\', "/");
    let mut vhost = format!(
        r#"# BoxedCat auto-generated vhost: {project_name}
server {{
    listen {port};
    server_name {server_name};
    root {root};

    location / {{
        try_files $uri $uri/ =404;
        index index.php index.html index.htm;
    }}

    location ~ \.php$ {{
        fastcgi_pass 127.0.0.1:9000;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }}

    access_log logs/{project_name}-access.log;
    error_log logs/{project_name}-error.log;
}}
"#
    );

    if ssl_enabled {
        vhost += &format!(
            r#"
server {{
    listen {ssl_port} ssl;
    server_name {server_name};
    root {root};

    ssl_certificate conf/ssl/{project_name}.crt;
    ssl_certificate_key conf/ssl/{project_name}.key;

    location / {{
        try_files $uri $uri/ =404;
        index index.php index.html index.htm;
    }}

    access_log logs/{project_name}-ssl-access.log;
    error_log logs/{project_name}-ssl-error.log;
}}
"#
        );
    }

    vhost
}

/// Discover projects in a directory.
pub fn discover_projects(www_dir: &Path) -> Vec<ProjectInfo> {
    let mut projects = Vec::new();
    if !www_dir.exists() {
        return projects;
    }

    if let Ok(entries) = std::fs::read_dir(www_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                let path = entry.path();
                let has_index = ["index.php", "index.html", "index.htm", "app.py", "main.ts"]
                    .iter()
                    .any(|f| path.join(f).exists());
                let server_name = format!("{}.test", name);
                projects.push(ProjectInfo { name, path, has_index, server_name });
            }
        }
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));
    projects
}

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub path: std::path::PathBuf,
    pub has_index: bool,
    pub server_name: String,
}
