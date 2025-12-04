use eframe::egui;
use std::collections::HashMap;

#[derive(Clone, Debug)]
enum NodeType {
    Variable,
    Function,
    Operation,
    Literal,
    Print,
}

#[derive(Clone)]
struct Node {
    id: u32,
    node_type: NodeType,
    position: (f32, f32),
    size: (f32, f32),
    properties: HashMap<String, String>,
}

struct VisualEditor {
    nodes: Vec<Node>,
    next_node_id: u32,
    pan: (f32, f32),
    zoom: f32,
    selected_node: Option<u32>,
    show_properties: bool,
    show_context_menu: bool,
    context_menu_pos: (f32, f32),
}

impl Default for VisualEditor {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            next_node_id: 1,
            pan: (0.0, 0.0),
            zoom: 1.0,
            selected_node: None,
            show_properties: false,
            show_context_menu: false,
            context_menu_pos: (0.0, 0.0),
        }
    }
}

impl VisualEditor {
    fn add_node(&mut self, node_type: NodeType, x: f32, y: f32) {
        let node = match node_type {
            NodeType::Variable => Node {
                id: self.next_node_id,
                node_type: NodeType::Variable,
                position: (x, y),
                size: (150.0, 80.0),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("name".to_string(), "x".to_string());
                    props.insert("type".to_string(), "i32".to_string());
                    props.insert("value".to_string(), "0".to_string());
                    props
                },
            },
            NodeType::Operation => Node {
                id: self.next_node_id,
                node_type: NodeType::Operation,
                position: (x, y),
                size: (150.0, 80.0),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("operator".to_string(), "+".to_string());
                    props
                },
            },
            NodeType::Literal => Node {
                id: self.next_node_id,
                node_type: NodeType::Literal,
                position: (x, y),
                size: (150.0, 60.0),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("value".to_string(), "0".to_string());
                    props.insert("type".to_string(), "i32".to_string());
                    props
                },
            },
            NodeType::Print => Node {
                id: self.next_node_id,
                node_type: NodeType::Print,
                position: (x, y),
                size: (150.0, 60.0),
                properties: HashMap::new(),
            },
            NodeType::Function => Node {
                id: self.next_node_id,
                node_type: NodeType::Function,
                position: (x, y),
                size: (180.0, 100.0),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("name".to_string(), "my_func".to_string());
                    props
                },
            },
        };
        
        self.next_node_id += 1;
        self.nodes.push(node);
    }
}

impl eframe::App for VisualEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Меню
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.nodes.clear();
                        self.next_node_id = 1;
                    }
                    if ui.button("Save").clicked() {
                        // TODO: Save project
                    }
                    if ui.button("Load").clicked() {
                        // TODO: Load project
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                
                ui.menu_button("Add Node", |ui| {
                    if ui.button("Variable").clicked() {
                        self.add_node(NodeType::Variable, 100.0, 100.0);
                    }
                    if ui.button("Operation").clicked() {
                        self.add_node(NodeType::Operation, 100.0, 100.0);
                    }
                    if ui.button("Literal").clicked() {
                        self.add_node(NodeType::Literal, 100.0, 100.0);
                    }
                    if ui.button("Print").clicked() {
                        self.add_node(NodeType::Print, 100.0, 100.0);
                    }
                    if ui.button("Function").clicked() {
                        self.add_node(NodeType::Function, 100.0, 100.0);
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_properties, "Properties");
                });
            });
        });
        
        // Основная область
        egui::CentralPanel::default().show(ctx, |ui| {
            // Фон
            let painter = ui.painter();
            let rect = ui.available_rect_before_wrap();
            painter.rect_filled(rect, 0.0, egui::Color32::from_gray(30));
            
            // Обработка перемещения холста
            if ui.input(|i| i.pointer.middle_down()) {
                self.pan.0 += ui.input(|i| i.pointer.delta().x);
                self.pan.1 += ui.input(|i| i.pointer.delta().y);
            }
            
            // Масштабирование
            if ui.input(|i| i.zoom_delta() != 1.0) {
                self.zoom *= ui.input(|i| i.zoom_delta());
                self.zoom = self.zoom.clamp(0.2, 5.0);
            }
            
            // Обработка правого клика для контекстного меню
            if ui.input(|i| i.pointer.secondary_clicked()) {
                if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                    self.context_menu_pos = (pointer_pos.x, pointer_pos.y);
                    self.show_context_menu = true;
                }
            }
            
            // Закрытие контекстного меню при клике вне его
            if ui.input(|i| i.pointer.primary_clicked()) && self.show_context_menu {
                self.show_context_menu = false;
            }
            
            // Собираем информацию о перетаскивании
            let mut dragged_node_id = None;
            let mut drag_delta = (0.0, 0.0);
            let mut clicked_node_id = None;
            
            // Проход 1: только рисование и сбор информации о перетаскивании
            for node in &self.nodes {
                let pos = egui::pos2(
                    node.position.0 * self.zoom + self.pan.0 + rect.center().x,
                    node.position.1 * self.zoom + self.pan.1 + rect.center().y,
                );
                
                let size = egui::vec2(node.size.0 * self.zoom, node.size.1 * self.zoom);
                let node_rect = egui::Rect::from_min_size(pos, size);
                
                // Фон узла
                let bg_color = if self.selected_node == Some(node.id) {
                    egui::Color32::from_rgb(80, 80, 120)
                } else {
                    egui::Color32::from_rgb(60, 60, 80)
                };
                
                painter.rect_filled(node_rect, 10.0 * self.zoom, bg_color);
                
                // Рамка узла
                painter.rect_stroke(
                    node_rect,
                    10.0 * self.zoom,
                    egui::Stroke::new(2.0 * self.zoom, egui::Color32::from_gray(100)),
                );
                
                // Текст узла
                let label = match node.node_type {
                    NodeType::Variable => "Variable",
                    NodeType::Function => "Function",
                    NodeType::Operation => "Operation",
                    NodeType::Literal => "Literal",
                    NodeType::Print => "Print",
                };
                
                painter.text(
                    pos + egui::vec2(10.0 * self.zoom, 20.0 * self.zoom),
                    egui::Align2::LEFT_TOP,
                    label,
                    egui::FontId::proportional(14.0 * self.zoom),
                    egui::Color32::WHITE,
                );
                
                // Дополнительная информация о узле
                match node.node_type {
                    NodeType::Variable => {
                        if let Some(name) = node.properties.get("name") {
                            painter.text(
                                pos + egui::vec2(10.0 * self.zoom, 40.0 * self.zoom),
                                egui::Align2::LEFT_TOP,
                                format!("Name: {}", name),
                                egui::FontId::proportional(12.0 * self.zoom),
                                egui::Color32::from_gray(200),
                            );
                        }
                    }
                    NodeType::Operation => {
                        if let Some(op) = node.properties.get("operator") {
                            painter.text(
                                pos + egui::vec2(10.0 * self.zoom, 40.0 * self.zoom),
                                egui::Align2::LEFT_TOP,
                                format!("Op: {}", op),
                                egui::FontId::proportional(12.0 * self.zoom),
                                egui::Color32::from_gray(200),
                            );
                        }
                    }
                    _ => {}
                }
                
                // Обработка взаимодействия с узлом
                let response = ui.interact(node_rect, egui::Id::new(node.id), egui::Sense::drag());
                
                if response.dragged() {
                    dragged_node_id = Some(node.id);
                    drag_delta = (
                        ui.input(|i| i.pointer.delta().x) / self.zoom,
                        ui.input(|i| i.pointer.delta().y) / self.zoom,
                    );
                }
                
                if response.clicked() {
                    clicked_node_id = Some(node.id);
                }
            }
            
            // Проход 2: обработка перетаскивания
            if let Some(node_id) = dragged_node_id {
                if let Some(node_mut) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                    node_mut.position.0 += drag_delta.0;
                    node_mut.position.1 += drag_delta.1;
                }
            }
            
            // Обработка клика
            if let Some(node_id) = clicked_node_id {
                self.selected_node = Some(node_id);
                self.show_properties = true;
            }
            
            // Контекстное меню для добавления узлов
            if self.show_context_menu {
                egui::Area::new(egui::Id::new("context_menu_area"))
                    .fixed_pos(egui::pos2(self.context_menu_pos.0, self.context_menu_pos.1))
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        egui::Frame::popup(ui.style()).show(ui, |ui| {
                            ui.set_min_width(150.0);
                            
                            if ui.button("Add Variable").clicked() {
                                let world_pos = (
                                    (self.context_menu_pos.0 - self.pan.0 - rect.center().x) / self.zoom,
                                    (self.context_menu_pos.1 - self.pan.1 - rect.center().y) / self.zoom,
                                );
                                self.add_node(NodeType::Variable, world_pos.0, world_pos.1);
                                self.show_context_menu = false;
                            }
                            if ui.button("Add Operation").clicked() {
                                let world_pos = (
                                    (self.context_menu_pos.0 - self.pan.0 - rect.center().x) / self.zoom,
                                    (self.context_menu_pos.1 - self.pan.1 - rect.center().y) / self.zoom,
                                );
                                self.add_node(NodeType::Operation, world_pos.0, world_pos.1);
                                self.show_context_menu = false;
                            }
                            if ui.button("Add Literal").clicked() {
                                let world_pos = (
                                    (self.context_menu_pos.0 - self.pan.0 - rect.center().x) / self.zoom,
                                    (self.context_menu_pos.1 - self.pan.1 - rect.center().y) / self.zoom,
                                );
                                self.add_node(NodeType::Literal, world_pos.0, world_pos.1);
                                self.show_context_menu = false;
                            }
                            if ui.button("Add Print").clicked() {
                                let world_pos = (
                                    (self.context_menu_pos.0 - self.pan.0 - rect.center().x) / self.zoom,
                                    (self.context_menu_pos.1 - self.pan.1 - rect.center().y) / self.zoom,
                                );
                                self.add_node(NodeType::Print, world_pos.0, world_pos.1);
                                self.show_context_menu = false;
                            }
                            if ui.button("Add Function").clicked() {
                                let world_pos = (
                                    (self.context_menu_pos.0 - self.pan.0 - rect.center().x) / self.zoom,
                                    (self.context_menu_pos.1 - self.pan.1 - rect.center().y) / self.zoom,
                                );
                                self.add_node(NodeType::Function, world_pos.0, world_pos.1);
                                self.show_context_menu = false;
                            }
                        });
                    });
            }
            
            // Панель информации
            egui::Window::new("Editor Info")
                .default_size((250.0, 120.0))
                .default_pos(egui::pos2(10.0, 30.0))
                .collapsible(true)
                .show(ctx, |ui| {
                    ui.label(format!("Nodes: {}", self.nodes.len()));
                    ui.label(format!("Zoom: {:.2}x", self.zoom));
                    ui.label(format!("Pan: ({:.1}, {:.1})", self.pan.0, self.pan.1));
                    ui.separator();
                    ui.label("Controls:");
                    ui.label("• Middle drag: Pan");
                    ui.label("• Wheel: Zoom");
                    ui.label("• Right click: Add node");
                });
        });
        
        // Панель свойств выбранного узла
        if self.show_properties {
            if let Some(selected_id) = self.selected_node {
                // Сначала получаем данные о узле
                let node_data = if let Some(node) = self.nodes.iter().find(|n| n.id == selected_id) {
                    Some((node.id, node.node_type.clone(), node.position, node.properties.clone()))
                } else {
                    None
                };
                
                // Затем отображаем окно с данными
                if let Some((node_id, node_type, position, properties)) = node_data {
                    let mut temp_properties = properties.clone();
                    let mut should_delete = false;
                    
                    egui::Window::new("Node Properties")
                        .default_size((300.0, 200.0))
                        .default_pos(egui::pos2(10.0, 160.0))
                        .collapsible(true)
                        .show(ctx, |ui| {
                            ui.label(format!("Node ID: {}", node_id));
                            ui.label(format!("Type: {:?}", node_type));
                            ui.separator();
                            
                            match node_type {
                                NodeType::Variable => {
                                    ui.horizontal(|ui| {
                                        ui.label("Name:");
                                        let mut name = temp_properties.get("name").cloned().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut name).changed() {
                                            temp_properties.insert("name".to_string(), name);
                                        }
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("Type:");
                                        let mut type_str = temp_properties.get("type").cloned().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut type_str).changed() {
                                            temp_properties.insert("type".to_string(), type_str);
                                        }
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("Value:");
                                        let mut value = temp_properties.get("value").cloned().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut value).changed() {
                                            temp_properties.insert("value".to_string(), value);
                                        }
                                    });
                                }
                                NodeType::Operation => {
                                    ui.horizontal(|ui| {
                                        ui.label("Operator:");
                                        let mut op = temp_properties.get("operator").cloned().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut op).changed() {
                                            temp_properties.insert("operator".to_string(), op);
                                        }
                                    });
                                    
                                    ui.label("Available operators: +, -, *, /, %, &&, ||, ==, !=, <, >, <=, >=");
                                }
                                NodeType::Literal => {
                                    ui.horizontal(|ui| {
                                        ui.label("Value:");
                                        let mut value = temp_properties.get("value").cloned().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut value).changed() {
                                            temp_properties.insert("value".to_string(), value);
                                        }
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("Type:");
                                        let mut type_str = temp_properties.get("type").cloned().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut type_str).changed() {
                                            temp_properties.insert("type".to_string(), type_str);
                                        }
                                    });
                                }
                                NodeType::Function => {
                                    ui.horizontal(|ui| {
                                        ui.label("Name:");
                                        let mut name = temp_properties.get("name").cloned().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut name).changed() {
                                            temp_properties.insert("name".to_string(), name);
                                        }
                                    });
                                }
                                NodeType::Print => {
                                    ui.label("Print node - outputs value to console");
                                }
                            }
                            
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                                ui.label(format!("({:.1}, {:.1})", position.0, position.1));
                            });
                            
                            if ui.button("Delete Node").clicked() {
                                should_delete = true;
                            }
                        });
                    
                    // Применяем изменения свойств
                    if let Some(node) = self.nodes.iter_mut().find(|n| n.id == selected_id) {
                        node.properties = temp_properties;
                    }
                    
                    // Удаляем узел если нужно
                    if should_delete {
                        self.nodes.retain(|n| n.id != selected_id);
                        self.selected_node = None;
                        self.show_properties = false;
                    }
                }
            }
        }
        
        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("Aetos Visual Editor"),
        ..Default::default()
    };

    // Логируем запуск приложения
    eprintln!("Starting Aetos Visual Editor...");
    
    // Запускаем приложение
    match eframe::run_native(
        "Aetos Visual Editor",
        options,
        Box::new(|cc| {
            // Можно настроить стили или другие параметры здесь
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(VisualEditor::default())
        }),
    ) {
        Ok(_) => {
            eprintln!("Application closed successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("Application error: {:?}", e);
            Err(e)
        }
    }
}