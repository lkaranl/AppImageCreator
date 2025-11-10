use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use crate::AppImageMetadata;

const DESKTOP_ENTRY_TEMPLATE: &str = r#"[Desktop Entry]
Type=Application
Name={name}
Exec={exec}
Icon={icon_name}
Categories={categories}
Version={version}
Comment={comment}
Terminal=false
"#;

const APPSTREAM_METADATA_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>{app_id}</id>
  <name>{name}</name>
  <summary>{comment}</summary>
  <description>
    <p>{description}</p>
  </description>
  <launchable type="desktop-id">{desktop_file}</launchable>
  <metadata_license>{metadata_license}</metadata_license>
  <project_license>{license}</project_license>{url_section}
  <provides>
    <binary>{exec}</binary>
  </provides>
</component>
"#;

pub fn generate_appimage(metadata: &AppImageMetadata, output_path: &Path) -> io::Result<()> {
    // Criar diretório de trabalho temporário
    let package_name = metadata.name.to_lowercase().replace(" ", "-");
    let temp_dir = std::env::temp_dir().join(format!("appimage-{}", package_name));

    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }

    let work_dir = temp_dir.join("project");
    fs::create_dir_all(&work_dir)?;

    // Criar estrutura do projeto
    let src_dir = work_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    let assets_dir = work_dir.join("assets");
    fs::create_dir_all(&assets_dir)?;

    // Criar main.rs dummy (necessário para compilar)
    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, "fn main() {}\n")?;

    // Nome do ícone baseado no pacote
    let icon_ext = Path::new(&metadata.icon_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png");
    let icon_name = package_name.clone();

    // Criar estrutura usr
    let usr_dir = assets_dir.join("usr");
    fs::create_dir_all(&usr_dir)?;

    // Copiar ícone para assets/icon.png
    let icon_in_assets = assets_dir.join("icon.png");
    fs::copy(&metadata.icon_path, &icon_in_assets)?;

    // Copiar binário para usr/bin
    let bin_dir = usr_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;
    let final_binary = bin_dir.join(&metadata.exec);
    fs::copy(&metadata.binary_path, &final_binary)?;

    // Tornar executável
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&final_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&final_binary, perms)?;
    }

    // Copiar ícone para usr/share/icons
    let icon_dir = usr_dir.join("share/icons/hicolor/256x256/apps");
    fs::create_dir_all(&icon_dir)?;
    let icon_dest = icon_dir.join(format!("{}.{}", icon_name, icon_ext));
    fs::copy(&metadata.icon_path, &icon_dest)?;

    // Criar diretório de aplicações
    let apps_dir = usr_dir.join("share/applications");
    fs::create_dir_all(&apps_dir)?;

    // Criar arquivo .desktop
    let desktop_content = DESKTOP_ENTRY_TEMPLATE
        .replace("{name}", &metadata.name)
        .replace("{exec}", &metadata.exec)
        .replace("{icon_name}", &icon_name)
        .replace("{categories}", &metadata.categories)
        .replace("{version}", &metadata.version)
        .replace("{comment}", &metadata.comment);

    let desktop_file_name = format!("{}.desktop", icon_name);
    let desktop_path = apps_dir.join(&desktop_file_name);
    fs::write(&desktop_path, &desktop_content)?;

    // Criar diretório para metainfo
    let metainfo_dir = usr_dir.join("share/metainfo");
    fs::create_dir_all(&metainfo_dir)?;

    // Criar arquivo AppStream metadata
    // Usar formato org.{autor}.{nome} se autor fornecido, senão org.github.{nome}
    let app_id = if !metadata.author.is_empty() {
        let author_slug = metadata.author
            .to_lowercase()
            .replace(" ", "")
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();
        format!("org.{}.{}", author_slug, icon_name)
    } else {
        format!("org.github.{}", icon_name)
    };

    let description = if !metadata.comment.is_empty() {
        metadata.comment.clone()
    } else {
        format!("Aplicação {}", metadata.name)
    };

    let url_section = if !metadata.website.is_empty() {
        format!(
            "\n  <url type=\"homepage\">{}</url>\n  <url type=\"bugtracker\">{}/issues</url>",
            metadata.website, metadata.website
        )
    } else {
        String::new()
    };

    let metadata_license = "CC0-1.0";
    let project_license = if !metadata.license.is_empty() {
        &metadata.license
    } else {
        "GPL-3.0-or-later"
    };

    let appstream_content = APPSTREAM_METADATA_TEMPLATE
        .replace("{app_id}", &app_id)
        .replace("{name}", &metadata.name)
        .replace("{comment}", &description)
        .replace("{description}", &description)
        .replace("{desktop_file}", &desktop_file_name)
        .replace("{metadata_license}", metadata_license)
        .replace("{license}", project_license)
        .replace("{url_section}", &url_section)
        .replace("{exec}", &metadata.exec);

    // Usar .appdata.xml como no projeto que funciona
    let metainfo_file_name = format!("{}.appdata.xml", app_id);
    let metainfo_path = metainfo_dir.join(&metainfo_file_name);
    fs::write(&metainfo_path, appstream_content)?;

    // Criar Cargo.toml
    let cargo_toml = work_dir.join("Cargo.toml");
    let mut cargo_content = format!(
        r#"[package]
name = "{}"
version = "{}"
edition = "2021"
"#,
        package_name,
        if metadata.version.is_empty() {
            "1.0.0"
        } else {
            &metadata.version
        }
    );

    if !metadata.author.is_empty() {
        cargo_content.push_str(&format!("authors = [\"{}\"]\n", metadata.author));
    }

    if !metadata.comment.is_empty() {
        cargo_content.push_str(&format!("description = \"{}\"\n", metadata.comment));
    }

    cargo_content.push_str(
        r#"
[package.metadata.appimage]
assets = ["assets/usr"]

[profile.release]
opt-level = 3
lto = true
strip = true
"#,
    );

    fs::write(&cargo_toml, cargo_content)?;

    // Verificar se cargo-appimage está instalado
    let cargo_appimage_check = Command::new("cargo")
        .args(&["appimage", "--version"])
        .output();

    if !cargo_appimage_check.is_ok() {
        // Limpar diretório temporário
        let _ = fs::remove_dir_all(&temp_dir);

        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "cargo-appimage não está instalado!\n\nPara instalar: cargo install cargo-appimage"
        ));
    }

    // Executar cargo appimage
    println!("Executando cargo appimage em: {}", work_dir.display());
    let output = Command::new("cargo")
        .arg("appimage")
        .current_dir(&work_dir)
        .output()?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        let out_msg = String::from_utf8_lossy(&output.stdout);
        println!("STDOUT: {}", out_msg);
        println!("STDERR: {}", error_msg);

        // Limpar diretório temporário
        let _ = fs::remove_dir_all(&temp_dir);

        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Erro ao executar cargo appimage: {}", error_msg)
        ));
    }

    // Mostrar output do cargo appimage
    println!("Output do cargo appimage:");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Procurar pelo AppImage gerado em vários lugares
    let search_paths = vec![
        work_dir.clone(),
        work_dir.join("target"),
        work_dir.join("target").join("appimage"),
    ];

    let mut found_appimage = None;

    for search_path in &search_paths {
        if search_path.exists() {
            println!("Procurando em: {}", search_path.display());

            if let Ok(entries) = fs::read_dir(search_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    println!("  Encontrado: {}", path.display());

                    if path.extension().and_then(|s| s.to_str()) == Some("AppImage") {
                        found_appimage = Some(path);
                        break;
                    }
                }
            }
        }

        if found_appimage.is_some() {
            break;
        }
    }

    if let Some(appimage_file) = found_appimage {
        println!("AppImage encontrado: {}", appimage_file.display());

        // Mover para o destino final
        fs::copy(&appimage_file, output_path)?;

        // Limpar diretório temporário
        let _ = fs::remove_dir_all(&temp_dir);

        println!("AppImage gerado com sucesso em: {}", output_path.display());
        Ok(())
    } else {
        println!("Conteúdo do diretório de trabalho:");
        if let Ok(entries) = fs::read_dir(&work_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                println!("  {}", entry.path().display());
            }
        }

        // NÃO limpar para você poder investigar
        println!("Projeto mantido em: {} para investigação", work_dir.display());

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("AppImage não foi encontrado após a compilação. Projeto em: {}", work_dir.display())
        ))
    }
}
