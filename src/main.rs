mod appimage;

use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, Entry, FileChooserAction, FileChooserDialog,
    Label, Orientation, ResponseType, ScrolledWindow, Align, ProgressBar,
    CheckButton,
};
use gtk4::glib::{self, ControlFlow, SourceId};
use libadwaita as adw;
use libadwaita::prelude::*;
use adw::{ApplicationWindow, HeaderBar, PreferencesGroup, ActionRow, Clamp, Toast, ToastOverlay};
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;
use std::time::Duration;

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

#[derive(Debug, Clone, Default)]
struct AppState {
    metadata: AppImageMetadata,
    output_folder: Option<PathBuf>,
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
    let app_state = Rc::new(RefCell::new(AppState::default()));

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
    binary_entry.set_width_chars(30);
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
    icon_entry.set_width_chars(30);
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
    name_entry.set_width_chars(30);
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
    exec_entry.set_width_chars(30);
    exec_row.add_suffix(&exec_entry);
    exec_row.set_activatable_widget(Some(&exec_entry));
    basic_group.add(&exec_row);

    // Categorias - usando ActionRow expandível com CheckButtons
    let categories_row = adw::ExpanderRow::new();
    categories_row.set_title("Categorias");
    categories_row.set_subtitle("Selecione as categorias do aplicativo");

    // Lista de categorias comuns do FreeDesktop
    let category_options = vec![
        ("AudioVideo", "Áudio e Vídeo"),
        ("Audio", "Áudio"),
        ("Video", "Vídeo"),
        ("Development", "Desenvolvimento"),
        ("Education", "Educação"),
        ("Game", "Jogo"),
        ("Graphics", "Gráficos"),
        ("Network", "Rede"),
        ("Office", "Escritório"),
        ("Science", "Ciência"),
        ("Settings", "Configurações"),
        ("System", "Sistema"),
        ("Utility", "Utilitário"),
    ];

    let mut category_checks = Vec::new();

    for (cat_value, cat_label) in category_options {
        let check_row = ActionRow::new();
        check_row.set_title(cat_label);
        let check = CheckButton::new();
        check.set_valign(Align::Center);
        check_row.add_prefix(&check);
        check_row.set_activatable_widget(Some(&check));
        categories_row.add_row(&check_row);
        category_checks.push((cat_value, check));
    }

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
    version_entry.set_width_chars(30);
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
    comment_entry.set_width_chars(30);
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
    author_entry.set_width_chars(30);
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
    license_entry.set_width_chars(30);
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
    website_entry.set_width_chars(30);
    website_row.add_suffix(&website_entry);
    website_row.set_activatable_widget(Some(&website_entry));
    details_group.add(&website_row);

    content_box.append(&details_group);

    // === GRUPO 4: Pasta de Saída ===
    let output_group = PreferencesGroup::new();
    output_group.set_title("Pasta de Saída");
    output_group.set_description(Some("Onde o AppImage será salvo"));

    let output_row = ActionRow::new();
    output_row.set_title("Pasta de destino");
    output_row.set_subtitle("Selecione onde salvar o AppImage");
    let output_entry = Entry::new();
    output_entry.set_placeholder_text(Some("Nenhuma pasta selecionada"));
    output_entry.set_editable(false);
    output_entry.set_valign(Align::Center);
    output_entry.set_hexpand(true);
    output_entry.set_width_chars(30);
    let output_button = Button::with_label("Escolher Pasta");
    output_button.set_valign(Align::Center);
    let output_box = Box::new(Orientation::Horizontal, 6);
    output_box.append(&output_entry);
    output_box.append(&output_button);
    output_row.add_suffix(&output_box);
    output_row.set_activatable_widget(Some(&output_button));
    output_group.add(&output_row);

    content_box.append(&output_group);

    scrolled.set_child(Some(&content_box));
    clamp.set_child(Some(&scrolled));
    main_box.append(&clamp);

    // Área do botão
    let button_box = Box::new(Orientation::Vertical, 8);
    button_box.set_margin_top(12);
    button_box.set_margin_bottom(18);
    button_box.set_margin_start(12);
    button_box.set_margin_end(12);

    // Botão com estrutura para progress
    let button_content = Box::new(Orientation::Vertical, 4);

    let button_label = Label::new(Some("Gerar AppImage"));
    button_label.set_margin_top(10);
    button_label.set_margin_bottom(4);
    button_content.append(&button_label);

    let progress_bar = ProgressBar::new();
    progress_bar.set_visible(false);
    progress_bar.set_margin_start(20);
    progress_bar.set_margin_end(20);
    progress_bar.set_margin_bottom(8);
    progress_bar.set_size_request(-1, 6); // Altura de 6px
    button_content.append(&progress_bar);

    let generate_button = Button::new();
    generate_button.set_child(Some(&button_content));
    generate_button.add_css_class("suggested-action");
    generate_button.add_css_class("pill");
    button_box.append(&generate_button);

    let pulse_source = Rc::new(RefCell::new(None::<SourceId>));
    let (result_sender, result_receiver) =
        glib::MainContext::channel::<Result<PathBuf, String>>(glib::Priority::default());

    main_box.append(&button_box);

    // Criar um box principal que contém headerbar e conteúdo
    let window_box = Box::new(Orientation::Vertical, 0);
    window_box.append(&header_bar);
    window_box.append(&main_box);

    // Toast overlay para notificações
    let toast_overlay = ToastOverlay::new();
    toast_overlay.set_child(Some(&window_box));

    window.set_content(Some(&toast_overlay));

    {
        let toast_clone = toast_overlay.clone();
        let progress_bar_clone = progress_bar.clone();
        let button_clone = generate_button.clone();
        let button_label_clone = button_label.clone();
        let pulse_source_clone = pulse_source.clone();

        result_receiver.attach(None, move |result| {
            // Parar animação de pulso
            if let Some(source_id) = pulse_source_clone.borrow_mut().take() {
                source_id.remove();
            }

            // Restaurar estado do botão
            progress_bar_clone.set_visible(false);
            progress_bar_clone.set_fraction(0.0);
            button_label_clone.set_text("Gerar AppImage");
            button_clone.set_sensitive(true);

            match result {
                Ok(path) => {
                    let toast = Toast::new(&format!(
                        "AppImage gerado com sucesso em:\n{}",
                        path.display()
                    ));
                    toast.set_timeout(5);
                    toast_clone.add_toast(toast);
                }
                Err(err) => {
                    let toast = Toast::new(&format!("Erro: {}", err));
                    toast.set_timeout(8);
                    toast_clone.add_toast(toast);
                }
            }

            ControlFlow::Continue
        });
    }

    // File chooser para binário
    {
        let window_clone = window.clone();
        let entry_clone = binary_entry.clone();
        let state_clone = app_state.clone();
        binary_button.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(
                Some("Selecione o Binário"),
                Some(&window_clone),
                FileChooserAction::Open,
                &[("Cancelar", ResponseType::Cancel), ("Selecionar", ResponseType::Accept)],
            );

            let entry_clone2 = entry_clone.clone();
            let state_clone2 = state_clone.clone();
            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            entry_clone2.set_text(&path_str);
                            state_clone2.borrow_mut().metadata.binary_path = path_str;
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
        let state_clone = app_state.clone();
        icon_button.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(
                Some("Selecione o Ícone"),
                Some(&window_clone),
                FileChooserAction::Open,
                &[("Cancelar", ResponseType::Cancel), ("Selecionar", ResponseType::Accept)],
            );

            let entry_clone2 = entry_clone.clone();
            let state_clone2 = state_clone.clone();
            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            entry_clone2.set_text(&path_str);
                            state_clone2.borrow_mut().metadata.icon_path = path_str;
                        }
                    }
                }
                dialog.close();
            });

            dialog.show();
        });
    }

    // Folder chooser para pasta de saída
    {
        let window_clone = window.clone();
        let entry_clone = output_entry.clone();
        let state_clone = app_state.clone();
        output_button.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(
                Some("Escolher Pasta de Saída"),
                Some(&window_clone),
                FileChooserAction::SelectFolder,
                &[("Cancelar", ResponseType::Cancel), ("Selecionar", ResponseType::Accept)],
            );

            let entry_clone2 = entry_clone.clone();
            let state_clone2 = state_clone.clone();
            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            entry_clone2.set_text(&path_str);
                            state_clone2.borrow_mut().output_folder = Some(path);
                        }
                    }
                }
                dialog.close();
            });

            dialog.show();
        });
    }

    // Conectar mudanças nos campos de texto
    connect_entry_to_state(&name_entry, app_state.clone(), |s, v| s.metadata.name = v);
    connect_entry_to_state(&exec_entry, app_state.clone(), |s, v| s.metadata.exec = v);
    connect_entry_to_state(&version_entry, app_state.clone(), |s, v| s.metadata.version = v);
    connect_entry_to_state(&comment_entry, app_state.clone(), |s, v| s.metadata.comment = v);
    connect_entry_to_state(&author_entry, app_state.clone(), |s, v| s.metadata.author = v);
    connect_entry_to_state(&license_entry, app_state.clone(), |s, v| s.metadata.license = v);
    connect_entry_to_state(&website_entry, app_state.clone(), |s, v| s.metadata.website = v);

    // Conectar mudanças nos checkboxes de categorias
    for (cat_value, check) in category_checks {
        let state_clone = app_state.clone();
        let cat_value_owned = cat_value.to_string();
        check.connect_toggled(move |check_btn| {
            let mut state = state_clone.borrow_mut();
            let mut categories: Vec<String> = state.metadata.categories
                .split(';')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();

            if check_btn.is_active() {
                // Adicionar categoria se não existir
                if !categories.contains(&cat_value_owned) {
                    categories.push(cat_value_owned.clone());
                }
            } else {
                // Remover categoria
                categories.retain(|c| c != &cat_value_owned);
            }

            // Reconstruir string com ponto e vírgula
            state.metadata.categories = if categories.is_empty() {
                String::new()
            } else {
                format!("{};", categories.join(";"))
            };
        });
    }

    // Ação do botão gerar
    {
        let toast_clone = toast_overlay.clone();
        let progress_bar_clone = progress_bar.clone();
        let button_clone = generate_button.clone();
        let button_label_clone = button_label.clone();
        let sender_clone = result_sender.clone();
        let pulse_source_clone = pulse_source.clone();

        generate_button.connect_clicked(move |_| {
            let state_data = app_state.borrow().clone();
            let metadata_data = &state_data.metadata;

            // Validar pasta de saída primeiro
            if state_data.output_folder.is_none() {
                let toast = Toast::new("Erro: Selecione a pasta de saída!");
                toast_clone.add_toast(toast);
                return;
            }

            // Validação dos campos obrigatórios
            if metadata_data.binary_path.is_empty() {
                let toast = Toast::new("Erro: Selecione o binário!");
                toast_clone.add_toast(toast);
                return;
            }
            if metadata_data.icon_path.is_empty() {
                let toast = Toast::new("Erro: Selecione o ícone!");
                toast_clone.add_toast(toast);
                return;
            }
            if metadata_data.name.is_empty() {
                let toast = Toast::new("Erro: Preencha o nome!");
                toast_clone.add_toast(toast);
                return;
            }
            if metadata_data.exec.is_empty() {
                let toast = Toast::new("Erro: Preencha o comando exec!");
                toast_clone.add_toast(toast);
                return;
            }
            if metadata_data.categories.is_empty() {
                let toast = Toast::new("Erro: Preencha as categorias!");
                toast_clone.add_toast(toast);
                return;
            }

            // Construir caminho de saída
            let output_folder = state_data.output_folder.unwrap();
            let output_path = output_folder.join(format!("{}.AppImage", metadata_data.name));

            // Mostrar progress bar no botão com feedback visual melhorado
            button_clone.set_sensitive(false);
            button_label_clone.set_text("Gerando AppImage...");
            progress_bar_clone.set_visible(true);
            progress_bar_clone.set_fraction(0.0);

            // Limpar timeout anterior se existir
            if let Some(source_id) = pulse_source_clone.borrow_mut().take() {
                source_id.remove();
            }

            // Criar animação de pulso suave (100ms para movimento mais fluido)
            let progress_for_timeout = progress_bar_clone.clone();
            let pulse_source_for_timeout = pulse_source_clone.clone();
            let timeout_id =
                glib::timeout_add_local(Duration::from_millis(100), move || {
                    progress_for_timeout.pulse();
                    ControlFlow::Continue
                });
            pulse_source_for_timeout.borrow_mut().replace(timeout_id);

            let metadata_clone = metadata_data.clone();
            let sender_for_thread = sender_clone.clone();

            std::thread::spawn(move || {
                let result = appimage::generate_appimage(
                    &metadata_clone,
                    output_path.as_path(),
                )
                .map(|_| output_path)
                .map_err(|err| err.to_string());

                let _ = sender_for_thread.send(result);
            });
        });
    }

    window.present();
}

fn connect_entry_to_state<F>(entry: &Entry, state: Rc<RefCell<AppState>>, setter: F)
where
    F: Fn(&mut AppState, String) + 'static,
{
    entry.connect_changed(move |entry| {
        let text = entry.text().to_string();
        setter(&mut state.borrow_mut(), text);
    });
}
