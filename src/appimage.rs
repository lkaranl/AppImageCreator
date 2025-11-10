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
    // Criar diretório AppDir temporário
    let package_name = metadata.name.to_lowercase().replace(" ", "-");
    let temp_dir = std::env::temp_dir().join(format!("appimage-{}", package_name));

    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }

    let appdir = temp_dir.join("AppDir");
    fs::create_dir_all(&appdir)?;

    // Nome do ícone baseado no pacote
    let icon_ext = Path::new(&metadata.icon_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png");
    let icon_name = package_name.clone();

    // Criar estrutura usr
    let usr_dir = appdir.join("usr");
    fs::create_dir_all(&usr_dir)?;

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

    // Copiar ícone para a raiz do AppDir com o nome do app
    let root_icon = appdir.join(format!("{}.{}", icon_name, icon_ext));
    fs::copy(&metadata.icon_path, &root_icon)?;

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

    // Copiar .desktop para a raiz do AppDir
    let root_desktop = appdir.join(&desktop_file_name);
    fs::write(&root_desktop, &desktop_content)?;

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

    // Criar AppRun
    let apprun_content = format!(
        r#"#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${{SELF%/*}}

# Configurar variáveis de ambiente
export PATH="${{HERE}}/usr/bin:${{PATH}}"
export LD_LIBRARY_PATH="${{HERE}}/usr/lib:${{LD_LIBRARY_PATH}}"
export XDG_DATA_DIRS="${{HERE}}/usr/share:${{XDG_DATA_DIRS}}"

# Executar o binário
exec "${{HERE}}/usr/bin/{}" "$@"
"#,
        metadata.exec
    );

    let apprun_path = appdir.join("AppRun");
    fs::write(&apprun_path, apprun_content)?;

    // Tornar AppRun executável
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&apprun_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&apprun_path, perms)?;
    }

    // Usar appimagetool para gerar o AppImage
    let appimagetool_check = Command::new("appimagetool")
        .arg("--version")
        .output();

    if appimagetool_check.is_ok() {
        // Executar appimagetool
        let output = Command::new("appimagetool")
            .arg(&appdir)
            .arg(output_path)
            .output()?;

        // Limpar diretório temporário
        let _ = fs::remove_dir_all(&temp_dir);

        if output.status.success() {
            println!("AppImage gerado com sucesso em: {}", output_path.display());
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Erro ao executar appimagetool: {}", error_msg)
            ))
        }
    } else {
        // Limpar diretório temporário
        let _ = fs::remove_dir_all(&temp_dir);

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "appimagetool não está instalado!\n\n\
            Para instalar:\n\
            wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage\n\
            chmod +x appimagetool-x86_64.AppImage\n\
            sudo mv appimagetool-x86_64.AppImage /usr/local/bin/appimagetool"
        ))
    }
}
