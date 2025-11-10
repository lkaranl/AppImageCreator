mod appimage;

use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, Entry, FileChooserAction, FileChooserDialog,
    Label, Orientation, ResponseType, ScrolledWindow, Align, ProgressBar,
    CheckButton, CssProvider, Image,
};
use gtk4::glib::{self, ControlFlow, SourceId};
use gtk4::gdk::Display;
use libadwaita as adw;
use libadwaita::prelude::*;
use adw::{ApplicationWindow, HeaderBar, PreferencesGroup, ActionRow, Clamp, Toast, ToastOverlay, ExpanderRow};
use std::cell::{RefCell, Cell};
use std::rc::Rc;
use std::path::PathBuf;
use std::time::Duration;
use async_channel::unbounded;
use std::fs;

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

    // Carregar estilos CSS customizados
    load_css();

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(700)
        .default_height(600)
        .build();

    let css_provider = CssProvider::new();
    css_provider.load_from_data(
        "
        entry.error {
            border: 1px solid @error_color;
            border-radius: 6px;
        }
        entry.success {
            border: 1px solid @success_color;
            border-radius: 6px;
        }
        .preferences-row.error {
            border: 1px solid @error_color;
            border-radius: 6px;
        }
        .preferences-row.success {
            border: 1px solid @success_color;
            border-radius: 6px;
        }
        button.large-action {
            padding: 18px 28px;
            border-radius: 24px;
        }
        button.large-action .title-label {
            font-size: 17px;
            font-weight: 600;
        }
        button.large-action .subtitle-label {
            font-size: 12px;
            opacity: 0.85;
        }
        progressbar.compact-progress trough {
            min-height: 6px;
            border-radius: 12px;
        }
        progressbar.compact-progress progress {
            border-radius: 12px;
        }
        "
    );

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // Tentar configurar ícone da aplicação
    // Se o ícone estiver instalado no sistema como "appimage-creator"
    gtk4::Window::set_default_icon_name("appimage-creator");

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
    binary_row.set_tooltip_text(Some("Arquivo binário compilado da sua aplicação"));
    add_prefix_icon_to_action_row(&binary_row, "📦");
    let binary_entry = Entry::new();
    binary_entry.set_placeholder_text(Some("Ex: /home/usuario/Projetos/meu-app/target/release/meu-app"));
    binary_entry.set_valign(Align::Center);
    binary_entry.set_hexpand(true);
    binary_entry.set_width_chars(30);
    let binary_button = Button::new();
    let binary_button_box = Box::new(Orientation::Horizontal, 6);
    let binary_button_icon = Label::new(Some("🔍"));
    binary_button_icon.add_css_class("dim-label");
    let binary_button_label = Label::new(Some("Procurar"));
    binary_button_box.append(&binary_button_icon);
    binary_button_box.append(&binary_button_label);
    binary_button.set_child(Some(&binary_button_box));
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
    icon_row.set_tooltip_text(Some("Imagem exibida no lançador e no AppImage"));
    add_prefix_icon_to_action_row(&icon_row, "🖼️");
    let icon_entry = Entry::new();
    icon_entry.set_placeholder_text(Some("Ex: /home/usuario/Imagens/icon.png"));
    icon_entry.set_valign(Align::Center);
    icon_entry.set_hexpand(true);
    icon_entry.set_width_chars(30);
    let icon_button = Button::new();
    let icon_button_box = Box::new(Orientation::Horizontal, 6);
    let icon_button_icon = Label::new(Some("🖼️"));
    icon_button_icon.add_css_class("dim-label");
    let icon_button_label = Label::new(Some("Procurar"));
    icon_button_box.append(&icon_button_icon);
    icon_button_box.append(&icon_button_label);
    icon_button.set_child(Some(&icon_button_box));
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
    name_row.set_tooltip_text(Some("Nome amigável exibido ao usuário"));
    add_prefix_icon_to_action_row(&name_row, "📝");
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("Ex: Meu Aplicativo"));
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
    exec_row.set_tooltip_text(Some("Comando usado no .desktop para iniciar a aplicação"));
    add_prefix_icon_to_action_row(&exec_row, "▶️");
    let exec_entry = Entry::new();
    exec_entry.set_placeholder_text(Some("Ex: meu-app"));
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
    categories_row.set_tooltip_text(Some("Categorias do menu seguindo o padrão FreeDesktop"));
    add_prefix_icon_to_expander_row(&categories_row, "📂");

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
    version_row.set_tooltip_text(Some("Versão exibida no metadado do AppImage"));
    add_prefix_icon_to_action_row(&version_row, "🔖");
    let version_entry = Entry::new();
    version_entry.set_placeholder_text(Some("Ex: 1.2.3"));
    version_entry.set_valign(Align::Center);
    version_entry.set_hexpand(true);
    version_entry.set_width_chars(30);
    version_row.add_suffix(&version_entry);
    version_row.set_activatable_widget(Some(&version_entry));
    details_group.add(&version_row);

    // Descrição
    let comment_row = ActionRow::new();
    comment_row.set_title("Descrição");
    comment_row.set_tooltip_text(Some("Breve resumo exibido em lojas e menus"));
    add_prefix_icon_to_action_row(&comment_row, "💬");
    let comment_entry = Entry::new();
    comment_entry.set_placeholder_text(Some("Ex: Ferramenta para gerar AppImages"));
    comment_entry.set_valign(Align::Center);
    comment_entry.set_hexpand(true);
    comment_entry.set_width_chars(30);
    comment_row.add_suffix(&comment_entry);
    comment_row.set_activatable_widget(Some(&comment_entry));
    details_group.add(&comment_row);

    // Autor
    let author_row = ActionRow::new();
    author_row.set_title("Autor");
    author_row.set_tooltip_text(Some("Pessoa ou organização responsável pelo app"));
    add_prefix_icon_to_action_row(&author_row, "👤");
    let author_entry = Entry::new();
    author_entry.set_placeholder_text(Some("Ex: Karan Luciano"));
    author_entry.set_valign(Align::Center);
    author_entry.set_hexpand(true);
    author_entry.set_width_chars(30);
    author_row.add_suffix(&author_entry);
    author_row.set_activatable_widget(Some(&author_entry));
    details_group.add(&author_row);

    // Licença
    let license_row = ExpanderRow::new();
    license_row.set_title("Licença");
    license_row.set_subtitle("Selecione uma licença comum ou informe outra");
    license_row.set_tooltip_text(Some("Licença de distribuição do seu aplicativo"));
    add_prefix_icon_to_expander_row(&license_row, "📜");

    let license_options = vec![
        ("GPL-3.0-or-later", "GNU GPL 3.0 ou superior"),
        ("GPL-2.0-or-later", "GNU GPL 2.0 ou superior"),
        ("LGPL-3.0-or-later", "GNU LGPL 3.0 ou superior"),
        ("MIT", "MIT"),
        ("Apache-2.0", "Apache 2.0"),
        ("BSD-3-Clause", "BSD 3-Clause"),
        ("MPL-2.0", "Mozilla Public License 2.0"),
        ("Proprietary", "Proprietária"),
    ];

    let mut license_checks_vec: Vec<(String, CheckButton)> = Vec::new();

    for (value, label) in &license_options {
        let check_row = ActionRow::new();
        check_row.set_title(*label);
        let check = CheckButton::new();
        check.set_valign(Align::Center);
        check_row.add_prefix(&check);
        check_row.set_activatable_widget(Some(&check));
        license_row.add_row(&check_row);
        license_checks_vec.push((value.to_string(), check));
    }

    let custom_license_row = ActionRow::new();
    custom_license_row.set_title("Outra licença");
    let license_entry = Entry::new();
    license_entry.set_placeholder_text(Some("Ex: GPL-3.0-or-later"));
    license_entry.set_valign(Align::Center);
    license_entry.set_hexpand(true);
    license_entry.set_width_chars(30);
    custom_license_row.add_suffix(&license_entry);
    custom_license_row.set_activatable_widget(Some(&license_entry));
    license_row.add_row(&custom_license_row);

    details_group.add(&license_row);

    // Website
    let website_row = ActionRow::new();
    website_row.set_title("Website");
    website_row.set_tooltip_text(Some("Site oficial, repositório ou página de suporte"));
    add_prefix_icon_to_action_row(&website_row, "🌐");
    let website_entry = Entry::new();
    website_entry.set_placeholder_text(Some("Ex: https://meuapp.dev"));
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
    output_row.set_title("Pasta de Saída");
    output_row.set_subtitle("Onde o AppImage será salvo");
    output_row.set_tooltip_text(Some("Diretório onde o arquivo AppImage final será criado"));
    add_prefix_icon_to_action_row(&output_row, "📁");
    let output_entry = Entry::new();
    output_entry.set_placeholder_text(Some("Ex: /home/usuario/Distribuicoes"));
    output_entry.set_editable(false);
    output_entry.set_valign(Align::Center);
    output_entry.set_hexpand(true);
    output_entry.set_width_chars(30);
    let output_button = Button::new();
    let output_button_box = Box::new(Orientation::Horizontal, 6);
    let output_button_icon = Label::new(Some("📁"));
    output_button_icon.add_css_class("dim-label");
    let output_button_label = Label::new(Some("Escolher Pasta"));
    output_button_box.append(&output_button_icon);
    output_button_box.append(&output_button_label);
    output_button.set_child(Some(&output_button_box));
    output_button.set_valign(Align::Center);
    let output_box = Box::new(Orientation::Horizontal, 6);
    output_box.append(&output_entry);
    output_box.append(&output_button);
    output_row.add_suffix(&output_box);
    output_row.set_activatable_widget(Some(&output_button));
    output_group.add(&output_row);

    let preview_label = Label::new(Some("Preencha os campos para ver o preview."));
    preview_label.set_wrap(true);
    preview_label.set_halign(Align::Center);
    preview_label.set_margin_bottom(4);

    let time_label = Label::new(Some(""));
    time_label.set_wrap(true);
    time_label.set_halign(Align::Center);
    time_label.set_margin_bottom(12);

    let time_label_for_ui = time_label.clone();
    let app_state_for_ui = app_state.clone();
    let binary_entry_for_ui = binary_entry.clone();
    let icon_entry_for_ui = icon_entry.clone();
    let name_entry_for_ui = name_entry.clone();
    let exec_entry_for_ui = exec_entry.clone();
    let categories_row_for_ui = categories_row.clone();
    let output_entry_for_ui = output_entry.clone();
    let preview_label_for_ui = preview_label.clone();

    let update_ui: Rc<dyn Fn()> = Rc::new(move || {
        let state = app_state_for_ui.borrow();
        set_widget_validation(&binary_entry_for_ui, !state.metadata.binary_path.is_empty());
        set_widget_validation(&icon_entry_for_ui, !state.metadata.icon_path.is_empty());
        set_widget_validation(&name_entry_for_ui, !state.metadata.name.is_empty());
        set_widget_validation(&exec_entry_for_ui, !state.metadata.exec.is_empty());
        set_widget_validation(&categories_row_for_ui, !state.metadata.categories.is_empty());
        set_widget_validation(&output_entry_for_ui, state.output_folder.is_some());

        if state.metadata.name.is_empty() || state.metadata.binary_path.is_empty() {
            preview_label_for_ui.set_text("Preencha o binário e o nome para ver o preview.");
            return;
        }

        let file_name = format!("{}.AppImage", state.metadata.name);
        let mut total_size: u64 = 0;
        if let Ok(meta) = fs::metadata(&state.metadata.binary_path) {
            total_size = total_size.saturating_add(meta.len());
        }
        if let Ok(meta) = fs::metadata(&state.metadata.icon_path) {
            total_size = total_size.saturating_add(meta.len());
        }
        if total_size > 0 {
            // Estimar overhead adicional de 5 MB para estrutura AppImage
            total_size = total_size.saturating_add(5 * 1024 * 1024);
            preview_label_for_ui.set_text(&format!(
                "Preview: {} (≈ {})",
                file_name,
                format_size(total_size)
            ));

            let estimated_secs = ((total_size as f64) / (1.5 * 1024.0 * 1024.0)).ceil() as u64;
            time_label_for_ui.set_text(&format!(
                "Estimativa de tempo: {}",
                format_duration(estimated_secs)
            ));
        } else {
            preview_label_for_ui.set_text(&format!("Preview: {}", file_name));
            time_label_for_ui.set_text("");
        }
    });

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
    let button_content = Box::new(Orientation::Vertical, 8);

    let header_box = Box::new(Orientation::Horizontal, 12);
    header_box.set_halign(Align::Center);

    let button_icon = Image::from_icon_name("system-run-symbolic");
    button_icon.set_pixel_size(28);
    header_box.append(&button_icon);

    let text_box = Box::new(Orientation::Vertical, 2);
    let button_label = Label::new(Some("Gerar AppImage"));
    button_label.add_css_class("title-label");
    let button_subtitle = Label::new(Some("Empacotar aplicação em formato portátil"));
    button_subtitle.add_css_class("subtitle-label");
    text_box.append(&button_label);
    text_box.append(&button_subtitle);
    header_box.append(&text_box);

    button_content.append(&header_box);

    let progress_bar = ProgressBar::new();
    progress_bar.add_css_class("compact-progress");
    progress_bar.set_visible(false);
    progress_bar.set_margin_top(4);
    button_content.append(&progress_bar);

    button_box.append(&preview_label);
    button_box.append(&time_label);

    let generate_button = Button::new();
    generate_button.set_child(Some(&button_content));
    generate_button.add_css_class("suggested-action");
    generate_button.add_css_class("pill");
    generate_button.add_css_class("large-action");
    button_box.append(&generate_button);

    let pulse_source = Rc::new(RefCell::new(None::<SourceId>));
    let (result_sender, result_receiver) = unbounded::<Result<PathBuf, String>>();

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
        let button_subtitle_clone = button_subtitle.clone();
        let pulse_source_clone = pulse_source.clone();

        glib::MainContext::default().spawn_local(async move {
            while let Ok(result) = result_receiver.recv().await {
                // Parar animação de pulso
                if let Some(source_id) = pulse_source_clone.borrow_mut().take() {
                    source_id.remove();
                }

                // Restaurar estado do botão
                progress_bar_clone.remove_css_class("pulsing");
                progress_bar_clone.set_visible(false);
                progress_bar_clone.set_fraction(0.0);
                button_label_clone.set_text("Gerar AppImage");
                button_subtitle_clone.set_text("Empacotar aplicação em formato portátil");
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
            }
        });
    }

    // File chooser para binário
    {
        let window_clone = window.clone();
        let entry_clone = binary_entry.clone();
        let state_clone = app_state.clone();
        let update_validation_clone = update_ui.clone();
        binary_button.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(
                Some("Selecione o Binário"),
                Some(&window_clone),
                FileChooserAction::Open,
                &[("Cancelar", ResponseType::Cancel), ("Selecionar", ResponseType::Accept)],
            );

            let entry_clone2 = entry_clone.clone();
            let state_clone2 = state_clone.clone();
            let update_validation_inner = update_validation_clone.clone();
            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            entry_clone2.set_text(&path_str);
                            state_clone2.borrow_mut().metadata.binary_path = path_str;
                            update_validation_inner.as_ref()();
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
        let update_validation_clone = update_ui.clone();
        icon_button.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(
                Some("Selecione o Ícone"),
                Some(&window_clone),
                FileChooserAction::Open,
                &[("Cancelar", ResponseType::Cancel), ("Selecionar", ResponseType::Accept)],
            );

            let entry_clone2 = entry_clone.clone();
            let state_clone2 = state_clone.clone();
            let update_validation_inner = update_validation_clone.clone();
            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            entry_clone2.set_text(&path_str);
                            state_clone2.borrow_mut().metadata.icon_path = path_str;
                            update_validation_inner.as_ref()();
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
        let update_validation_clone = update_ui.clone();
        output_button.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(
                Some("Escolher Pasta de Saída"),
                Some(&window_clone),
                FileChooserAction::SelectFolder,
                &[("Cancelar", ResponseType::Cancel), ("Selecionar", ResponseType::Accept)],
            );

            let entry_clone2 = entry_clone.clone();
            let state_clone2 = state_clone.clone();
            let update_validation_inner = update_validation_clone.clone();
            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            entry_clone2.set_text(&path_str);
                            state_clone2.borrow_mut().output_folder = Some(path);
                            update_validation_inner.as_ref()();
                        }
                    }
                }
                dialog.close();
            });

            dialog.show();
        });
    }

    // Conectar mudanças nos campos de texto
    connect_entry_to_state(
        &binary_entry,
        app_state.clone(),
        |s, v| s.metadata.binary_path = v,
        update_ui.clone(),
    );
    connect_entry_to_state(
        &icon_entry,
        app_state.clone(),
        |s, v| s.metadata.icon_path = v,
        update_ui.clone(),
    );
    connect_entry_to_state(
        &name_entry,
        app_state.clone(),
        |s, v| s.metadata.name = v,
        update_ui.clone(),
    );
    connect_entry_to_state(
        &exec_entry,
        app_state.clone(),
        |s, v| s.metadata.exec = v,
        update_ui.clone(),
    );
    connect_entry_to_state(
        &version_entry,
        app_state.clone(),
        |s, v| s.metadata.version = v,
        update_ui.clone(),
    );
    connect_entry_to_state(
        &comment_entry,
        app_state.clone(),
        |s, v| s.metadata.comment = v,
        update_ui.clone(),
    );
    connect_entry_to_state(
        &author_entry,
        app_state.clone(),
        |s, v| s.metadata.author = v,
        update_ui.clone(),
    );
    connect_entry_to_state(
        &website_entry,
        app_state.clone(),
        |s, v| s.metadata.website = v,
        update_ui.clone(),
    );

    let license_checks = Rc::new(license_checks_vec);
    let license_update_flag = Rc::new(Cell::new(false));

    // Conectar mudanças nos checkboxes de categorias
    let update_ui_for_categories = update_ui.clone();
    for (cat_value, check) in category_checks {
        let state_clone = app_state.clone();
        let cat_value_owned = cat_value.to_string();
        let update_ui_local = update_ui_for_categories.clone();
        check.connect_toggled(move |check_btn| {
            {
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
            }

            update_ui_local.as_ref()();
        });
    }

    update_ui.as_ref()();

    // Conectar seleção de licença
    {
        let license_checks_for_loop = license_checks.clone();
        let update_ui_for_license = update_ui.clone();
        for (value, check) in license_checks_for_loop.iter() {
            let value_owned = value.clone();
            let check_clone = check.clone();
            let checks_clone = license_checks.clone();
            let entry_clone = license_entry.clone();
            let state_clone = app_state.clone();
            let update_flag = license_update_flag.clone();
            let update_ui_local = update_ui_for_license.clone();

            check_clone.connect_toggled(move |btn| {
                if update_flag.get() {
                    return;
                }

                update_flag.set(true);

                {
                    let mut state = state_clone.borrow_mut();

                    if btn.is_active() {
                        for (other_value, other_check) in checks_clone.iter() {
                            if other_value != &value_owned {
                                other_check.set_active(false);
                            }
                        }
                        entry_clone.set_text(&value_owned);
                        state.metadata.license = value_owned.clone();
                    } else if checks_clone.iter().all(|(_, c)| !c.is_active()) {
                        entry_clone.set_text("");
                        state.metadata.license.clear();
                    }
                }

                update_ui_local.as_ref()();
                update_flag.set(false);
            });
        }

        let checks_for_entry = license_checks.clone();
        let state_for_entry = app_state.clone();
        let update_flag_entry = license_update_flag.clone();
        let update_ui_for_entry = update_ui.clone();

        license_entry.connect_changed(move |entry| {
            if update_flag_entry.get() {
                return;
            }

            update_flag_entry.set(true);

            {
                let text = entry.text().to_string();
                for (_, check) in checks_for_entry.iter() {
                    if check.is_active() {
                        check.set_active(false);
                    }
                }

                state_for_entry.borrow_mut().metadata.license = text;
            }

            update_ui_for_entry.as_ref()();
            update_flag_entry.set(false);
        });
    }

    // Ação do botão gerar
    {
        let toast_clone = toast_overlay.clone();
        let progress_bar_clone = progress_bar.clone();
        let button_clone = generate_button.clone();
        let button_label_clone = button_label.clone();
        let button_subtitle_clone = button_subtitle.clone();
        let sender_clone = result_sender.clone();
        let pulse_source_clone = pulse_source.clone();

        generate_button.connect_clicked(move |_| {
            let state_data = app_state.borrow().clone();
            let metadata_data = &state_data.metadata;

            // Validar pasta de saída primeiro
            if state_data.output_folder.is_none() {
                let toast = Toast::new("Atenção: Selecione a pasta de saída!");
                toast_clone.add_toast(toast);
                return;
            }

            // Validação dos campos obrigatórios
            if metadata_data.binary_path.is_empty() {
                let toast = Toast::new("Atenção: Selecione o binário!");
                toast_clone.add_toast(toast);
                return;
            }
            if metadata_data.icon_path.is_empty() {
                let toast = Toast::new("Atenção: Selecione o ícone!");
                toast_clone.add_toast(toast);
                return;
            }
            if metadata_data.name.is_empty() {
                let toast = Toast::new("Atenção: Preencha o nome!");
                toast_clone.add_toast(toast);
                return;
            }
            if metadata_data.exec.is_empty() {
                let toast = Toast::new("Atenção: Preencha o comando exec!");
                toast_clone.add_toast(toast);
                return;
            }
            if metadata_data.categories.is_empty() {
                let toast = Toast::new("Atenção: Preencha as categorias!");
                toast_clone.add_toast(toast);
                return;
            }

            // Construir caminho de saída
            let output_folder = state_data.output_folder.unwrap();
            let output_path = output_folder.join(format!("{}.AppImage", metadata_data.name));

            // Mostrar progress bar no botão com feedback visual melhorado
            button_clone.set_sensitive(false);
            button_label_clone.set_text("Gerando AppImage...");
            button_subtitle_clone.set_text("Isso pode levar alguns instantes");
            progress_bar_clone.set_visible(true);
            progress_bar_clone.set_fraction(0.0);
            progress_bar_clone.add_css_class("pulsing");

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

                let _ = sender_for_thread.send_blocking(result);
            });
        });
    }

    window.present();
}

fn connect_entry_to_state<F>(
    entry: &Entry,
    state: Rc<RefCell<AppState>>,
    setter: F,
    on_change: Rc<dyn Fn()>,
)
where
    F: Fn(&mut AppState, String) + 'static,
{
    let on_change_clone = on_change.clone();
    entry.connect_changed(move |entry| {
        let text = entry.text().to_string();
        setter(&mut state.borrow_mut(), text);
        on_change_clone.as_ref()();
    });
}

fn add_prefix_icon_to_action_row(row: &ActionRow, emoji: &str) {
    let icon_label = Label::new(Some(emoji));
    icon_label.add_css_class("dim-label");
    icon_label.set_margin_end(8);
    icon_label.set_margin_start(4);
    row.add_prefix(&icon_label);
}

fn add_prefix_icon_to_expander_row(row: &ExpanderRow, emoji: &str) {
    let icon_label = Label::new(Some(emoji));
    icon_label.add_css_class("dim-label");
    icon_label.set_margin_end(8);
    icon_label.set_margin_start(4);
    row.add_prefix(&icon_label);
}

fn set_widget_validation<W: gtk4::prelude::WidgetExt>(widget: &W, is_valid: bool) {
    widget.remove_css_class("error");
    widget.remove_css_class("success");

    if is_valid {
        widget.add_css_class("success");
    } else {
        widget.add_css_class("error");
    }
}

fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let size = bytes as f64;
    if size >= GB {
        format!("{:.2} GB", size / GB)
    } else if size >= MB {
        format!("{:.1} MB", size / MB)
    } else if size >= KB {
        format!("{:.0} KB", size / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn format_duration(seconds: u64) -> String {
    if seconds >= 60 {
        let minutes = seconds / 60;
        let remaining = seconds % 60;
        if remaining == 0 {
            format!("{} min", minutes)
        } else {
            format!("{} min {} s", minutes, remaining)
        }
    } else {
        format!("{} s", seconds)
    }
}

fn load_css() {
    let css = r#"
        /* Espaçamento consistente entre grupos */
        preferencesgroup {
            margin-top: 16px;
            margin-bottom: 16px;
        }

        preferencesgroup:first-child {
            margin-top: 12px;
        }

        preferencesgroup:last-child {
            margin-bottom: 12px;
        }

        /* Progress bar com bordas arredondadas e estilo melhorado */
        progressbar trough {
            min-height: 6px;
            border-radius: 6px;
            background-color: alpha(@window_fg_color, 0.15);
        }

        progressbar progress {
            min-height: 6px;
            border-radius: 6px;
            background-color: @accent_bg_color;
            box-shadow: 0 1px 3px alpha(black, 0.2);
        }

        /* Melhorar espaçamento dos ActionRow */
        row {
            padding: 8px 0;
        }

        /* Estilo para o botão principal */
        button.suggested-action.pill {
            min-height: 44px;
            font-weight: bold;
        }

        /* Animação suave para progress bar */
        @keyframes pulse {
            0% { opacity: 0.8; }
            50% { opacity: 1.0; }
            100% { opacity: 0.8; }
        }

        progressbar.pulsing progress {
            animation: pulse 1.5s ease-in-out infinite;
        }
    "#;

    let provider = CssProvider::new();
    provider.load_from_data(css);

    // Aplicar CSS ao display padrão
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
