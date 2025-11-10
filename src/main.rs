mod appimage;

use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, Entry, FileChooserAction, FileChooserDialog,
    Label, Orientation, ResponseType, ScrolledWindow, Align,
};
use libadwaita as adw;
use libadwaita::prelude::*;
use adw::{ApplicationWindow, HeaderBar, PreferencesGroup, ActionRow, Clamp};
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "com.github.appimage-creator";

#[derive(Debug, Clone, Default)]
struct AppImageMetadata {
    binary_path: String,
    icon_path: String,
    name: String,
    exec: String,
    categories: String,
    version: String,
    comment: String,
    author: String,
    license: String,
    website: String,
}

fn main() {
    adw::init().expect("Falha ao inicializar libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let metadata = Rc::new(RefCell::new(AppImageMetadata::default()));

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(700)
        .default_height(600)
        .build();

    // Criar HeaderBar
    let header_bar = HeaderBar::new();
    header_bar.set_title_widget(Some(&Label::new(Some("AppImage Creator"))));
    header_bar.set_show_end_title_buttons(true);
    header_bar.set_show_start_title_buttons(true);

    // Container principal com Clamp para largura máxima
    let clamp = Clamp::new();
    clamp.set_maximum_size(700);
    clamp.set_tightening_threshold(600);

    let main_box = Box::new(Orientation::Vertical, 0);

    // Scrolled window
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let content_box = Box::new(Orientation::Vertical, 24);
    content_box.set_margin_top(24);
    content_box.set_margin_bottom(24);
    content_box.set_margin_start(12);
    content_box.set_margin_end(12);

    // === GRUPO 1: Arquivos ===
    let files_group = PreferencesGroup::new();
    files_group.set_title("Arquivos");
    files_group.set_description(Some("Selecione o binário e ícone da aplicação"));

    // Binário
    let binary_row = ActionRow::new();
    binary_row.set_title("Binário");
    binary_row.set_subtitle("Executável da aplicação");
    let binary_entry = Entry::new();
    binary_entry.set_placeholder_text(Some("Selecione o executável"));
    binary_entry.set_valign(Align::Center);
    binary_entry.set_hexpand(true);
    let binary_button = Button::with_label("Procurar");
    binary_button.set_valign(Align::Center);
    let binary_box = Box::new(Orientation::Horizontal, 6);
    binary_box.append(&binary_entry);
    binary_box.append(&binary_button);
    binary_row.add_suffix(&binary_box);
    binary_row.set_activatable_widget(Some(&binary_button));
    files_group.add(&binary_row);

    // Ícone
    let icon_row = ActionRow::new();
    icon_row.set_title("Ícone");
    icon_row.set_subtitle("Imagem do ícone (PNG, JPG, etc)");
    let icon_entry = Entry::new();
    icon_entry.set_placeholder_text(Some("Selecione a imagem"));
    icon_entry.set_valign(Align::Center);
    icon_entry.set_hexpand(true);
    let icon_button = Button::with_label("Procurar");
    icon_button.set_valign(Align::Center);
    let icon_box = Box::new(Orientation::Horizontal, 6);
    icon_box.append(&icon_entry);
    icon_box.append(&icon_button);
    icon_row.add_suffix(&icon_box);
    icon_row.set_activatable_widget(Some(&icon_button));
    files_group.add(&icon_row);

    content_box.append(&files_group);

    // === GRUPO 2: Informações Básicas ===
    let basic_group = PreferencesGroup::new();
    basic_group.set_title("Informações Básicas");
    basic_group.set_description(Some("Dados essenciais da aplicação"));

    // Nome
    let name_row = ActionRow::new();
    name_row.set_title("Nome");
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("Nome da aplicação"));
    name_entry.set_valign(Align::Center);
    name_entry.set_hexpand(true);
    name_row.add_suffix(&name_entry);
    name_row.set_activatable_widget(Some(&name_entry));
    basic_group.add(&name_row);

    // Exec
    let exec_row = ActionRow::new();
    exec_row.set_title("Comando");
    exec_row.set_subtitle("Nome do executável (ex: myapp)");
    let exec_entry = Entry::new();
    exec_entry.set_placeholder_text(Some("myapp"));
    exec_entry.set_valign(Align::Center);
    exec_entry.set_hexpand(true);
    exec_row.add_suffix(&exec_entry);
    exec_row.set_activatable_widget(Some(&exec_entry));
    basic_group.add(&exec_row);

    // Categorias
    let categories_row = ActionRow::new();
    categories_row.set_title("Categorias");
    categories_row.set_subtitle("Separadas por ponto e vírgula");
    let categories_entry = Entry::new();
    categories_entry.set_placeholder_text(Some("Utility;Development;"));
    categories_entry.set_valign(Align::Center);
    categories_entry.set_hexpand(true);
    categories_row.add_suffix(&categories_entry);
    categories_row.set_activatable_widget(Some(&categories_entry));
    basic_group.add(&categories_row);

    content_box.append(&basic_group);

    // === GRUPO 3: Detalhes (Opcional) ===
    let details_group = PreferencesGroup::new();
    details_group.set_title("Detalhes");
    details_group.set_description(Some("Informações adicionais (opcional)"));

    // Versão
    let version_row = ActionRow::new();
    version_row.set_title("Versão");
    let version_entry = Entry::new();
    version_entry.set_placeholder_text(Some("1.0.0"));
    version_entry.set_valign(Align::Center);
    version_entry.set_hexpand(true);
    version_row.add_suffix(&version_entry);
    version_row.set_activatable_widget(Some(&version_entry));
    details_group.add(&version_row);

    // Descrição
    let comment_row = ActionRow::new();
    comment_row.set_title("Descrição");
    let comment_entry = Entry::new();
    comment_entry.set_placeholder_text(Some("Breve descrição da aplicação"));
    comment_entry.set_valign(Align::Center);
    comment_entry.set_hexpand(true);
    comment_row.add_suffix(&comment_entry);
    comment_row.set_activatable_widget(Some(&comment_entry));
    details_group.add(&comment_row);

    // Autor
    let author_row = ActionRow::new();
    author_row.set_title("Autor");
    let author_entry = Entry::new();
    author_entry.set_placeholder_text(Some("Seu nome"));
    author_entry.set_valign(Align::Center);
    author_entry.set_hexpand(true);
    author_row.add_suffix(&author_entry);
    author_row.set_activatable_widget(Some(&author_entry));
    details_group.add(&author_row);

    // Licença
    let license_row = ActionRow::new();
    license_row.set_title("Licença");
    let license_entry = Entry::new();
    license_entry.set_placeholder_text(Some("MIT, GPL, Apache, etc."));
    license_entry.set_valign(Align::Center);
    license_entry.set_hexpand(true);
    license_row.add_suffix(&license_entry);
    license_row.set_activatable_widget(Some(&license_entry));
    details_group.add(&license_row);

    // Website
    let website_row = ActionRow::new();
    website_row.set_title("Website");
    let website_entry = Entry::new();
    website_entry.set_placeholder_text(Some("https://exemplo.com"));
    website_entry.set_valign(Align::Center);
    website_entry.set_hexpand(true);
    website_row.add_suffix(&website_entry);
    website_row.set_activatable_widget(Some(&website_entry));
    details_group.add(&website_row);

    content_box.append(&details_group);

    scrolled.set_child(Some(&content_box));
    clamp.set_child(Some(&scrolled));
    main_box.append(&clamp);

    // Área do botão e status
    let button_box = Box::new(Orientation::Vertical, 12);
    button_box.set_margin_top(12);
    button_box.set_margin_bottom(18);
    button_box.set_margin_start(12);
    button_box.set_margin_end(12);

    let generate_button = Button::builder()
        .label("Gerar AppImage")
        .height_request(48)
        .build();
    generate_button.add_css_class("suggested-action");
    generate_button.add_css_class("pill");
    button_box.append(&generate_button);

    // Status label
    let status_label = Label::new(None);
    status_label.set_wrap(true);
    status_label.set_wrap_mode(gtk4::pango::WrapMode::Word);
    status_label.set_max_width_chars(60);
    status_label.add_css_class("dim-label");
    button_box.append(&status_label);

    main_box.append(&button_box);

    // Criar um box principal que contém headerbar e conteúdo
    let window_box = Box::new(Orientation::Vertical, 0);
    window_box.append(&header_bar);
    window_box.append(&main_box);

    window.set_content(Some(&window_box));

    // File chooser para binário
    {
        let window_clone = window.clone();
        let entry_clone = binary_entry.clone();
        let metadata_clone = metadata.clone();
        binary_button.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(
                Some("Selecione o Binário"),
                Some(&window_clone),
                FileChooserAction::Open,
                &[("Cancelar", ResponseType::Cancel), ("Selecionar", ResponseType::Accept)],
            );

            let entry_clone2 = entry_clone.clone();
            let metadata_clone2 = metadata_clone.clone();
            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            entry_clone2.set_text(&path_str);
                            metadata_clone2.borrow_mut().binary_path = path_str;
                        }
                    }
                }
                dialog.close();
            });

            dialog.show();
        });
    }

    // File chooser para ícone
    {
        let window_clone = window.clone();
        let entry_clone = icon_entry.clone();
        let metadata_clone = metadata.clone();
        icon_button.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(
                Some("Selecione o Ícone"),
                Some(&window_clone),
                FileChooserAction::Open,
                &[("Cancelar", ResponseType::Cancel), ("Selecionar", ResponseType::Accept)],
            );

            let entry_clone2 = entry_clone.clone();
            let metadata_clone2 = metadata_clone.clone();
            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            entry_clone2.set_text(&path_str);
                            metadata_clone2.borrow_mut().icon_path = path_str;
                        }
                    }
                }
                dialog.close();
            });

            dialog.show();
        });
    }

    // Conectar mudanças nos campos de texto
    connect_entry_to_metadata(&name_entry, metadata.clone(), |m, v| m.name = v);
    connect_entry_to_metadata(&exec_entry, metadata.clone(), |m, v| m.exec = v);
    connect_entry_to_metadata(&categories_entry, metadata.clone(), |m, v| m.categories = v);
    connect_entry_to_metadata(&version_entry, metadata.clone(), |m, v| m.version = v);
    connect_entry_to_metadata(&comment_entry, metadata.clone(), |m, v| m.comment = v);
    connect_entry_to_metadata(&author_entry, metadata.clone(), |m, v| m.author = v);
    connect_entry_to_metadata(&license_entry, metadata.clone(), |m, v| m.license = v);
    connect_entry_to_metadata(&website_entry, metadata.clone(), |m, v| m.website = v);

    // Ação do botão gerar
    {
        let window_clone = window.clone();
        let status_clone = status_label.clone();
        generate_button.connect_clicked(move |_| {
            let metadata_data = metadata.borrow().clone();

            // Validação
            if metadata_data.binary_path.is_empty() {
                status_clone.set_text("Erro: Selecione o binário!");
                return;
            }
            if metadata_data.icon_path.is_empty() {
                status_clone.set_text("Erro: Selecione o ícone!");
                return;
            }
            if metadata_data.name.is_empty() {
                status_clone.set_text("Erro: Preencha o nome!");
                return;
            }
            if metadata_data.exec.is_empty() {
                status_clone.set_text("Erro: Preencha o comando exec!");
                return;
            }
            if metadata_data.categories.is_empty() {
                status_clone.set_text("Erro: Preencha as categorias!");
                return;
            }

            status_clone.set_text("Gerando AppImage...");

            // Escolher onde salvar
            let dialog = FileChooserDialog::new(
                Some("Salvar AppImage"),
                Some(&window_clone),
                FileChooserAction::Save,
                &[("Cancelar", ResponseType::Cancel), ("Salvar", ResponseType::Accept)],
            );
            dialog.set_current_name(&format!("{}.AppImage", metadata_data.name));

            let status_clone2 = status_clone.clone();
            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            match appimage::generate_appimage(&metadata_data, &path) {
                                Ok(_) => {
                                    status_clone2.set_text(&format!(
                                        "AppImage gerado com sucesso em: {}",
                                        path.display()
                                    ));
                                }
                                Err(e) => {
                                    status_clone2.set_text(&format!("Erro ao gerar AppImage: {}", e));
                                }
                            }
                        }
                    }
                }
                dialog.close();
            });

            dialog.show();
        });
    }

    window.present();
}

fn connect_entry_to_metadata<F>(entry: &Entry, metadata: Rc<RefCell<AppImageMetadata>>, setter: F)
where
    F: Fn(&mut AppImageMetadata, String) + 'static,
{
    entry.connect_changed(move |entry| {
        let text = entry.text().to_string();
        setter(&mut metadata.borrow_mut(), text);
    });
}
