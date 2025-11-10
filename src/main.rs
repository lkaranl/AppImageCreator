mod appimage;

use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, Entry, FileChooserAction, FileChooserDialog,
    Label, Orientation, ResponseType, Grid, ScrolledWindow,
};
use libadwaita as adw;
use libadwaita::prelude::*;
use adw::{ApplicationWindow, HeaderBar};
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

    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);

    // Header
    let header_label = Label::new(Some("Gerador de AppImage"));
    header_label.add_css_class("title-1");
    main_box.append(&header_label);

    let subtitle_label = Label::new(Some("Preencha os campos para criar seu AppImage"));
    subtitle_label.add_css_class("dim-label");
    main_box.append(&subtitle_label);

    // Scrolled window para os campos
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(12);

    let mut row = 0;

    // Campos obrigatórios
    let required_label = Label::new(Some("Campos Obrigatórios"));
    required_label.add_css_class("title-3");
    required_label.set_halign(gtk4::Align::Start);
    grid.attach(&required_label, 0, row, 2, 1);
    row += 1;

    // Binário
    let (binary_label, binary_entry, binary_button) = create_file_row("Binário:", "Selecione o executável");
    grid.attach(&binary_label, 0, row, 1, 1);
    let binary_box = Box::new(Orientation::Horizontal, 6);
    binary_box.append(&binary_entry);
    binary_box.append(&binary_button);
    grid.attach(&binary_box, 1, row, 1, 1);
    row += 1;

    // Ícone
    let (icon_label, icon_entry, icon_button) = create_file_row("Ícone:", "Selecione a imagem");
    grid.attach(&icon_label, 0, row, 1, 1);
    let icon_box = Box::new(Orientation::Horizontal, 6);
    icon_box.append(&icon_entry);
    icon_box.append(&icon_button);
    grid.attach(&icon_box, 1, row, 1, 1);
    row += 1;

    // Nome
    let (name_label, name_entry) = create_text_row("Nome:", "Nome da aplicação");
    grid.attach(&name_label, 0, row, 1, 1);
    grid.attach(&name_entry, 1, row, 1, 1);
    row += 1;

    // Exec (comando)
    let (exec_label, exec_entry) = create_text_row("Exec:", "Comando para executar (ex: myapp)");
    grid.attach(&exec_label, 0, row, 1, 1);
    grid.attach(&exec_entry, 1, row, 1, 1);
    row += 1;

    // Categorias
    let (categories_label, categories_entry) = create_text_row("Categorias:", "Ex: Utility;Development;");
    grid.attach(&categories_label, 0, row, 1, 1);
    grid.attach(&categories_entry, 1, row, 1, 1);
    row += 1;

    // Campos opcionais
    row += 1;
    let optional_label = Label::new(Some("Campos Opcionais"));
    optional_label.add_css_class("title-3");
    optional_label.set_halign(gtk4::Align::Start);
    grid.attach(&optional_label, 0, row, 2, 1);
    row += 1;

    // Versão
    let (version_label, version_entry) = create_text_row("Versão:", "1.0.0");
    grid.attach(&version_label, 0, row, 1, 1);
    grid.attach(&version_entry, 1, row, 1, 1);
    row += 1;

    // Descrição/Comentário
    let (comment_label, comment_entry) = create_text_row("Descrição:", "Descrição da aplicação");
    grid.attach(&comment_label, 0, row, 1, 1);
    grid.attach(&comment_entry, 1, row, 1, 1);
    row += 1;

    // Autor
    let (author_label, author_entry) = create_text_row("Autor:", "Seu nome");
    grid.attach(&author_label, 0, row, 1, 1);
    grid.attach(&author_entry, 1, row, 1, 1);
    row += 1;

    // Licença
    let (license_label, license_entry) = create_text_row("Licença:", "MIT, GPL, etc.");
    grid.attach(&license_label, 0, row, 1, 1);
    grid.attach(&license_entry, 1, row, 1, 1);
    row += 1;

    // Website
    let (website_label, website_entry) = create_text_row("Website:", "https://exemplo.com");
    grid.attach(&website_label, 0, row, 1, 1);
    grid.attach(&website_entry, 1, row, 1, 1);

    scrolled.set_child(Some(&grid));
    main_box.append(&scrolled);

    // Botão de gerar
    let generate_button = Button::builder()
        .label("Gerar AppImage")
        .margin_top(12)
        .build();
    generate_button.add_css_class("suggested-action");
    generate_button.add_css_class("pill");

    main_box.append(&generate_button);

    // Status label
    let status_label = Label::new(None);
    status_label.set_margin_top(12);
    main_box.append(&status_label);

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

fn create_file_row(label_text: &str, placeholder: &str) -> (Label, Entry, Button) {
    let label = Label::new(Some(label_text));
    label.set_halign(gtk4::Align::Start);

    let entry = Entry::new();
    entry.set_placeholder_text(Some(placeholder));
    entry.set_hexpand(true);

    let button = Button::builder()
        .label("Procurar")
        .build();

    (label, entry, button)
}

fn create_text_row(label_text: &str, placeholder: &str) -> (Label, Entry) {
    let label = Label::new(Some(label_text));
    label.set_halign(gtk4::Align::Start);

    let entry = Entry::new();
    entry.set_placeholder_text(Some(placeholder));
    entry.set_hexpand(true);

    (label, entry)
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
