use eframe::egui;
use std::collections::HashMap;
use std::collections::HashSet;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
enum PortType {
    Input,
    Output,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Port {
    id: String,
    name: String,
    port_type: PortType,
    data_type: String,
    position: (f32, f32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Connection {
    id: u32,
    from_node: u32,
    from_port: String,
    to_node: u32,
    to_port: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
enum NodeType {
    Variable,
    Function,
    Operation,
    Literal,
    Print,
}

#[derive(Clone, Serialize, Deserialize)]
struct Node {
    id: u32,
    node_type: NodeType,
    position: (f32, f32),
    size: (f32, f32),
    properties: HashMap<String, String>,
    input_ports: Vec<Port>,
    output_ports: Vec<Port>,
}

#[derive(Serialize, Deserialize)]
struct VisualEditor {
    nodes: Vec<Node>,
    connections: Vec<Connection>,
    next_node_id: u32,
    next_connection_id: u32,
    pan: (f32, f32),
    zoom: f32,
    selected_node: Option<u32>,
    selected_connection: Option<u32>,
    dragging_node: Option<u32>,
    #[serde(skip)]
    dragging_connection_start: Option<(u32, String, egui::Pos2)>,
    show_properties: bool,
    show_context_menu: bool,
    context_menu_pos: (f32, f32),
    save_dialog_open: bool,
    load_dialog_open: bool,
    file_path: String,
    show_code_window: bool,
    show_info_window: bool,
}

impl Default for VisualEditor {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
            next_node_id: 1,
            next_connection_id: 1,
            pan: (0.0, 0.0),
            zoom: 1.0,
            selected_node: None,
            selected_connection: None,
            dragging_node: None,
            dragging_connection_start: None,
            show_properties: false,
            show_context_menu: false,
            context_menu_pos: (0.0, 0.0),
            save_dialog_open: false,
            load_dialog_open: false,
            file_path: String::new(),
            show_code_window: true,
            show_info_window: true,
        }
    }
}

impl VisualEditor {
    fn add_node(&mut self, node_type: NodeType, x: f32, y: f32) {
        let (input_ports, output_ports) = match node_type {
            NodeType::Variable => {
                let output_port = Port {
                    id: "value".to_string(),
                    name: "value".to_string(),
                    port_type: PortType::Output,
                    data_type: "i32".to_string(),
                    position: (150.0, 40.0),
                };
                (Vec::new(), vec![output_port])
            }
            NodeType::Operation => {
                let input1 = Port {
                    id: "left".to_string(),
                    name: "A".to_string(),
                    port_type: PortType::Input,
                    data_type: "i32".to_string(),
                    position: (0.0, 20.0),
                };
                let input2 = Port {
                    id: "right".to_string(),
                    name: "B".to_string(),
                    port_type: PortType::Input,
                    data_type: "i32".to_string(),
                    position: (0.0, 60.0),
                };
                let output = Port {
                    id: "result".to_string(),
                    name: "Result".to_string(),
                    port_type: PortType::Output,
                    data_type: "i32".to_string(),
                    position: (150.0, 40.0),
                };
                (vec![input1, input2], vec![output])
            }
            NodeType::Literal => {
                let output = Port {
                    id: "value".to_string(),
                    name: "value".to_string(),
                    port_type: PortType::Output,
                    data_type: "i32".to_string(),
                    position: (150.0, 30.0),
                };
                (Vec::new(), vec![output])
            }
            NodeType::Print => {
                let input = Port {
                    id: "value".to_string(),
                    name: "value".to_string(),
                    port_type: PortType::Input,
                    data_type: "i32".to_string(),
                    position: (0.0, 30.0),
                };
                (vec![input], Vec::new())
            }
            NodeType::Function => {
                let input1 = Port {
                    id: "param1".to_string(),
                    name: "x".to_string(),
                    port_type: PortType::Input,
                    data_type: "i32".to_string(),
                    position: (0.0, 20.0),
                };
                let input2 = Port {
                    id: "param2".to_string(),
                    name: "y".to_string(),
                    port_type: PortType::Input,
                    data_type: "i32".to_string(),
                    position: (0.0, 50.0),
                };
                let output = Port {
                    id: "result".to_string(),
                    name: "result".to_string(),
                    port_type: PortType::Output,
                    data_type: "i32".to_string(),
                    position: (180.0, 35.0),
                };
                (vec![input1, input2], vec![output])
            }
        };
        
        let node = match node_type {
            NodeType::Variable => Node {
                id: self.next_node_id,
                node_type: NodeType::Variable,
                position: (x, y),
                size: (150.0, 80.0),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("name".to_string(), format!("var_{}", self.next_node_id));
                    props.insert("type".to_string(), "i32".to_string());
                    props.insert("value".to_string(), "0".to_string());
                    props
                },
                input_ports,
                output_ports,
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
                input_ports,
                output_ports,
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
                input_ports,
                output_ports,
            },
            NodeType::Print => Node {
                id: self.next_node_id,
                node_type: NodeType::Print,
                position: (x, y),
                size: (150.0, 60.0),
                properties: HashMap::new(),
                input_ports,
                output_ports,
            },
            NodeType::Function => Node {
                id: self.next_node_id,
                node_type: NodeType::Function,
                position: (x, y),
                size: (180.0, 100.0),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("name".to_string(), format!("func_{}", self.next_node_id));
                    props
                },
                input_ports,
                output_ports,
            },
        };
        
        self.next_node_id += 1;
        self.nodes.push(node);
    }
    
    fn draw_connections(&self, painter: &egui::Painter, rect: egui::Rect) {
        for connection in &self.connections {
            if let (Some(from_node), Some(to_node)) = (
                self.nodes.iter().find(|n| n.id == connection.from_node),
                self.nodes.iter().find(|n| n.id == connection.to_node),
            ) {
                if let (Some(from_port), Some(to_port)) = (
                    from_node.output_ports.iter().find(|p| p.id == connection.from_port),
                    to_node.input_ports.iter().find(|p| p.id == connection.to_port),
                ) {
                    let from_pos = self.port_screen_position(from_node, from_port, rect);
                    let to_pos = self.port_screen_position(to_node, to_port, rect);
                    
                    self.draw_bezier_curve(painter, from_pos, to_pos, connection.id);
                }
            }
        }
        
        if let Some((node_id, port_id, start_pos)) = &self.dragging_connection_start {
            if let Some(node) = self.nodes.iter().find(|n| n.id == *node_id) {
                if let Some(port) = node.output_ports.iter().find(|p| p.id == *port_id) {
                    let port_pos = self.port_screen_position(node, port, rect);
                    let stroke = egui::Stroke::new(2.0 * self.zoom, egui::Color32::from_rgb(255, 200, 100));
                    painter.line_segment([port_pos, *start_pos], stroke);
                    painter.circle_filled(*start_pos, 5.0 * self.zoom, egui::Color32::from_rgb(255, 200, 100));
                }
            }
        }
    }
    
    fn port_screen_position(&self, node: &Node, port: &Port, rect: egui::Rect) -> egui::Pos2 {
        let x_offset = match port.port_type {
            PortType::Input => port.position.0,
            PortType::Output => node.size.0 - (node.size.0 - port.position.0),
        };
        
        egui::pos2(
            node.position.0 * self.zoom + self.pan.0 + rect.center().x + x_offset * self.zoom,
            node.position.1 * self.zoom + self.pan.1 + rect.center().y + port.position.1 * self.zoom,
        )
    }
    
    fn draw_bezier_curve(&self, painter: &egui::Painter, from: egui::Pos2, to: egui::Pos2, connection_id: u32) {
        let mid_x = (from.x + to.x) / 2.0;
        let control1 = egui::pos2(mid_x, from.y);
        let control2 = egui::pos2(mid_x, to.y);
        
        let stroke = if self.selected_connection == Some(connection_id) {
            egui::Stroke::new(3.0 * self.zoom, egui::Color32::from_rgb(255, 200, 50))
        } else {
            egui::Stroke::new(2.0 * self.zoom, egui::Color32::from_rgb(100, 200, 100))
        };
        
        let points = vec![from, control1, control2, to];
        painter.add(egui::Shape::line(points, stroke));
        
        // Рисуем стрелку
        let arrow_size = 8.0 * self.zoom;
        let arrow_dir = 0.5;
        
        let arrow1 = to - egui::vec2(arrow_dir * arrow_size, arrow_dir * arrow_size);
        let arrow2 = to - egui::vec2(arrow_dir * arrow_size, -arrow_dir * arrow_size);
        
        painter.line_segment([to, arrow1], stroke);
        painter.line_segment([to, arrow2], stroke);
    }
    
    fn draw_ports(&self, painter: &egui::Painter, node: &Node, node_rect: egui::Rect) {
        for port in &node.input_ports {
            let port_pos = egui::pos2(
                node_rect.left() + port.position.0 * self.zoom,
                node_rect.top() + port.position.1 * self.zoom,
            );
            
            painter.circle_filled(
                port_pos,
                5.0 * self.zoom,
                egui::Color32::from_rgb(100, 150, 255),
            );
            
            painter.text(
                port_pos + egui::vec2(8.0 * self.zoom, 4.0 * self.zoom),
                egui::Align2::LEFT_CENTER,
                &port.name,
                egui::FontId::proportional(10.0 * self.zoom),
                egui::Color32::from_gray(200),
            );
        }
        
        for port in &node.output_ports {
            let port_pos = egui::pos2(
                node_rect.right() - (node.size.0 - port.position.0) * self.zoom,
                node_rect.top() + port.position.1 * self.zoom,
            );
            
            painter.circle_filled(
                port_pos,
                5.0 * self.zoom,
                egui::Color32::from_rgb(255, 150, 100),
            );
            
            painter.text(
                port_pos - egui::vec2(8.0 * self.zoom, 4.0 * self.zoom),
                egui::Align2::RIGHT_CENTER,
                &port.name,
                egui::FontId::proportional(10.0 * self.zoom),
                egui::Color32::from_gray(200),
            );
        }
    }
    
    fn check_port_click(&mut self, node: &Node, node_rect: egui::Rect, mouse_pos: egui::Pos2) -> bool {
        for port in &node.output_ports {
            let port_pos = egui::pos2(
                node_rect.right() - (node.size.0 - port.position.0) * self.zoom,
                node_rect.top() + port.position.1 * self.zoom,
            );
            
            let port_circle = egui::Rect::from_center_size(port_pos, egui::Vec2::splat(10.0 * self.zoom));
            
            if port_circle.contains(mouse_pos) {
                self.dragging_connection_start = Some((node.id, port.id.clone(), mouse_pos));
                return true;
            }
        }
        
        for port in &node.input_ports {
            let port_pos = egui::pos2(
                node_rect.left() + port.position.0 * self.zoom,
                node_rect.top() + port.position.1 * self.zoom,
            );
            
            let port_circle = egui::Rect::from_center_size(port_pos, egui::Vec2::splat(10.0 * self.zoom));
            
            if port_circle.contains(mouse_pos) {
                if let Some((from_node_id, from_port_id, _)) = &self.dragging_connection_start {
                    if *from_node_id != node.id {
                        if self.can_connect_ports(from_node_id, from_port_id, &node.id, &port.id) {
                            let connection = Connection {
                                id: self.next_connection_id,
                                from_node: *from_node_id,
                                from_port: from_port_id.clone(),
                                to_node: node.id,
                                to_port: port.id.clone(),
                            };
                            
                            self.connections.push(connection);
                            self.next_connection_id += 1;
                        }
                    }
                    self.dragging_connection_start = None;
                    return true;
                }
                return true;
            }
        }
        
        false
    }
    
    fn can_connect_ports(&self, from_node_id: &u32, from_port_id: &str, to_node_id: &u32, to_port_id: &str) -> bool {
        if from_node_id == to_node_id {
            return false;
        }
        
        if let (Some(from_node), Some(to_node)) = (
            self.nodes.iter().find(|n| n.id == *from_node_id),
            self.nodes.iter().find(|n| n.id == *to_node_id),
        ) {
            if let (Some(from_port), Some(to_port)) = (
                from_node.output_ports.iter().find(|p| p.id == from_port_id),
                to_node.input_ports.iter().find(|p| p.id == to_port_id),
            ) {
                return from_port.data_type == to_port.data_type;
            }
        }
        
        false
    }
    
    fn generate_code(&self) -> String {
        let mut code = String::new();
        let mut variable_declarations = Vec::new();
        let mut statements = Vec::new();
        
        for node in &self.nodes {
            match node.node_type {
                NodeType::Variable => {
                    if let (Some(name), Some(var_type), Some(value)) = (
                        node.properties.get("name"),
                        node.properties.get("type"),
                        node.properties.get("value")
                    ) {
                        let value_expr = self.get_input_expression(node.id, "value")
                            .unwrap_or_else(|| value.clone());
                        
                        variable_declarations.push(format!("let {}: {} = {};", name, var_type, value_expr));
                    }
                }
                NodeType::Literal => {
                    if let Some(value) = node.properties.get("value") {
                        if let Some(_var_name) = self.find_variable_using_node(node.id) {
                            // Уже используется в переменной
                        } else {
                            let temp_var = format!("temp_{}", node.id);
                            variable_declarations.push(format!("let {} = {};", temp_var, value));
                        }
                    }
                }
                NodeType::Operation => {
                    let left_expr = self.get_input_expression(node.id, "left").unwrap_or_else(|| "0".to_string());
                    let right_expr = self.get_input_expression(node.id, "right").unwrap_or_else(|| "0".to_string());
                    
                    if let Some(operator) = node.properties.get("operator") {
                        let expr = format!("{} {} {}", left_expr, operator, right_expr);
                        
                        if let Some(var_name) = self.find_variable_using_node(node.id) {
                            statements.push(format!("{} = {};", var_name, expr));
                        } else if self.is_node_used(node.id) {
                            let temp_var = format!("op_{}", node.id);
                            variable_declarations.push(format!("let {} = {};", temp_var, expr));
                        }
                    }
                }
                NodeType::Print => {
                    if let Some(value_expr) = self.get_input_expression(node.id, "value") {
                        statements.push(format!("print({});", value_expr));
                    }
                }
                NodeType::Function => {
                    if let Some(func_name) = node.properties.get("name") {
                        let mut args = Vec::new();
                        
                        for port in &node.input_ports {
                            if let Some(arg_expr) = self.get_input_expression(node.id, &port.id) {
                                args.push(arg_expr);
                            } else {
                                args.push("0".to_string());
                            }
                        }
                        
                        let call_expr = format!("{}({})", func_name, args.join(", "));
                        
                        if let Some(var_name) = self.find_variable_using_node(node.id) {
                            statements.push(format!("{} = {};", var_name, call_expr));
                        } else if self.is_node_used(node.id) {
                            let temp_var = format!("call_{}", node.id);
                            variable_declarations.push(format!("let {} = {};", temp_var, call_expr));
                        } else {
                            statements.push(format!("{};", call_expr));
                        }
                    }
                }
            }
        }
        
        code.push_str(&variable_declarations.join("\n"));
        if !variable_declarations.is_empty() && !statements.is_empty() {
            code.push('\n');
        }
        code.push_str(&statements.join("\n"));
        
        if code.trim().is_empty() {
            code = "// No code generated".to_string();
        }
        
        format!("fn main() -> i32 {{\n    {}\n    0\n}}", 
            code.lines().map(|line| format!("    {}", line)).collect::<Vec<_>>().join("\n    "))
    }
    
    fn get_input_expression(&self, node_id: u32, port_id: &str) -> Option<String> {
        if let Some(connection) = self.find_connection_to_input(node_id, port_id) {
            if let Some(source_node) = self.nodes.iter().find(|n| n.id == connection.from_node) {
                match source_node.node_type {
                    NodeType::Variable => {
                        return source_node.properties.get("name").cloned();
                    }
                    NodeType::Literal => {
                        return source_node.properties.get("value").cloned();
                    }
                    NodeType::Operation => {
                        let left = self.get_input_expression(source_node.id, "left").unwrap_or_else(|| "0".to_string());
                        let right = self.get_input_expression(source_node.id, "right").unwrap_or_else(|| "0".to_string());
                        if let Some(operator) = source_node.properties.get("operator") {
                            return Some(format!("{} {} {}", left, operator, right));
                        }
                    }
                    NodeType::Function => {
                        if let Some(func_name) = source_node.properties.get("name") {
                            let mut args = Vec::new();
                            for port in &source_node.input_ports {
                                args.push(self.get_input_expression(source_node.id, &port.id).unwrap_or_else(|| "0".to_string()));
                            }
                            return Some(format!("{}({})", func_name, args.join(", ")));
                        }
                    }
                    _ => {}
                }
            }
        }
        
        None
    }
    
    fn find_connection_to_input(&self, node_id: u32, port_id: &str) -> Option<&Connection> {
        self.connections.iter()
            .find(|c| c.to_node == node_id && c.to_port == port_id)
    }
    
    fn find_variable_using_node(&self, node_id: u32) -> Option<String> {
        for node in &self.nodes {
            if node.node_type == NodeType::Variable {
                if let Some(value) = node.properties.get("value") {
                    if value == &format!("node_{}", node_id) {
                        return node.properties.get("name").cloned();
                    }
                }
            }
        }
        None
    }
    
    fn is_node_used(&self, node_id: u32) -> bool {
        self.connections.iter()
            .any(|c| c.from_node == node_id)
    }
    
    fn export_project(&self) -> Result<String, Box<dyn std::error::Error>> {
        let export_data = serde_json::to_string_pretty(self)?;
        Ok(export_data)
    }
    
    fn import_project(&mut self, json_content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let imported: VisualEditor = serde_json::from_str(json_content)?;
        
        self.nodes = imported.nodes;
        self.connections = imported.connections;
        self.next_node_id = imported.next_node_id;
        self.next_connection_id = imported.next_connection_id;
        self.pan = imported.pan;
        self.zoom = imported.zoom;
        self.selected_node = imported.selected_node;
        self.selected_connection = imported.selected_connection;
        self.show_properties = imported.show_properties;
        self.show_context_menu = imported.show_context_menu;
        self.context_menu_pos = imported.context_menu_pos;
        self.save_dialog_open = imported.save_dialog_open;
        self.load_dialog_open = imported.load_dialog_open;
        self.file_path = imported.file_path;
        self.show_code_window = imported.show_code_window;
        self.show_info_window = imported.show_info_window;
        
        // Поля с атрибутом #[serde(skip)] не загружаются, оставляем их по умолчанию
        // self.dragging_connection_start уже None по умолчанию
        // self.dragging_node уже None по умолчанию
        
        Ok(())
    }
    
    fn validate_connections(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        for connection in &self.connections {
            let from_exists = self.nodes.iter().any(|n| n.id == connection.from_node);
            let to_exists = self.nodes.iter().any(|n| n.id == connection.to_node);
            
            if !from_exists {
                errors.push(format!("Connection {}: Source node {} not found", connection.id, connection.from_node));
                continue;
            }
            if !to_exists {
                errors.push(format!("Connection {}: Target node {} not found", connection.id, connection.to_node));
                continue;
            }
            
            if let (Some(from_node), Some(to_node)) = (
                self.nodes.iter().find(|n| n.id == connection.from_node),
                self.nodes.iter().find(|n| n.id == connection.to_node),
            ) {
                let from_port_exists = from_node.output_ports.iter()
                    .any(|p| p.id == connection.from_port);
                let to_port_exists = to_node.input_ports.iter()
                    .any(|p| p.id == connection.to_port);
                
                if !from_port_exists {
                    errors.push(format!("Connection {}: Output port '{}' not found on node {}", 
                        connection.id, connection.from_port, from_node.id));
                }
                
                if !to_port_exists {
                    errors.push(format!("Connection {}: Input port '{}' not found on node {}", 
                        connection.id, connection.to_port, to_node.id));
                }
            }
        }
        
        errors
    }
    
    fn auto_layout(&mut self) {
        use std::collections::VecDeque;
        
        if self.nodes.is_empty() {
            return;
        }
        
        let mut layers: HashMap<u32, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        
        let roots: Vec<u32> = self.nodes.iter()
            .filter(|n| !self.connections.iter().any(|c| c.to_node == n.id))
            .map(|n| n.id)
            .collect();
        
        if roots.is_empty() {
            if let Some(first) = self.nodes.first() {
                layers.insert(first.id, 0);
                queue.push_back(first.id);
            }
        } else {
            for &root in &roots {
                layers.insert(root, 0);
                queue.push_back(root);
            }
        }
        
        while let Some(node_id) = queue.pop_front() {
            let current_layer = *layers.get(&node_id).unwrap_or(&0);
            
            for connection in &self.connections {
                if connection.from_node == node_id {
                    let next_id = connection.to_node;
                    let next_layer = current_layer + 1;
                    
                    if !layers.contains_key(&next_id) || layers[&next_id] < next_layer {
                        layers.insert(next_id, next_layer);
                        queue.push_back(next_id);
                    }
                }
            }
        }
        
        let mut nodes_by_layer: HashMap<usize, Vec<u32>> = HashMap::new();
        let max_layer = *layers.values().max().unwrap_or(&0);
        
        for (&node_id, &layer) in &layers {
            nodes_by_layer.entry(layer).or_default().push(node_id);
        }
        
        let horizontal_spacing = 200.0;
        let vertical_spacing = 100.0;
        let start_x = -100.0;
        
        for layer in 0..=max_layer {
            if let Some(layer_nodes) = nodes_by_layer.get(&layer) {
                let layer_height = (layer_nodes.len() as f32 - 1.0) * vertical_spacing;
                let start_y = -layer_height / 2.0;
                
                for (i, &node_id) in layer_nodes.iter().enumerate() {
                    if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                        node.position.0 = start_x + layer as f32 * horizontal_spacing;
                        node.position.1 = start_y + i as f32 * vertical_spacing;
                    }
                }
            }
        }
    }
    
    fn find_cycles(&self) -> Vec<Vec<u32>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        
        for node in &self.nodes {
            if !visited.contains(&node.id) {
                let mut stack = Vec::new();
                let mut in_stack = HashSet::new();
                
                self.dfs_find_cycle(node.id, &mut visited, &mut stack, &mut in_stack, &mut cycles);
            }
        }
        
        cycles
    }
    
    fn dfs_find_cycle(
        &self,
        node_id: u32,
        visited: &mut HashSet<u32>,
        stack: &mut Vec<u32>,
        in_stack: &mut HashSet<u32>,
        cycles: &mut Vec<Vec<u32>>
    ) {
        stack.push(node_id);
        in_stack.insert(node_id);
        
        for connection in &self.connections {
            if connection.from_node == node_id {
                let next_id = connection.to_node;
                
                if !visited.contains(&next_id) {
                    if !in_stack.contains(&next_id) {
                        self.dfs_find_cycle(next_id, visited, stack, in_stack, cycles);
                    } else {
                        let cycle_start = stack.iter().position(|&id| id == next_id).unwrap();
                        let cycle = stack[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }
        
        stack.pop();
        in_stack.remove(&node_id);
        visited.insert(node_id);
    }
    
    fn delete_node(&mut self, node_id: u32) {
        self.connections.retain(|c| c.from_node != node_id && c.to_node != node_id);
        self.nodes.retain(|n| n.id != node_id);
        if self.selected_node == Some(node_id) {
            self.selected_node = None;
            self.show_properties = false;
        }
    }
    
    fn delete_connection(&mut self, connection_id: u32) {
        self.connections.retain(|c| c.id != connection_id);
        if self.selected_connection == Some(connection_id) {
            self.selected_connection = None;
        }
    }
    
    fn duplicate_node(&mut self, node_id: u32) {
        if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
            let mut new_node = node.clone();
            new_node.id = self.next_node_id;
            new_node.position.0 += 20.0;
            new_node.position.1 += 20.0;
            
            if let Some(name) = new_node.properties.get_mut("name") {
                *name = format!("{}_copy", name);
            }
            
            self.next_node_id += 1;
            self.nodes.push(new_node);
        }
    }
}

impl eframe::App for VisualEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project").clicked() {
                        *self = VisualEditor::default();
                    }
                    if ui.button("Save Project").clicked() {
                        self.save_dialog_open = true;
                    }
                    if ui.button("Load Project").clicked() {
                        self.load_dialog_open = true;
                    }
                    ui.separator();
                    if ui.button("Generate Code").clicked() {
                        let code = self.generate_code();
                        println!("Generated Code:\n{}", code);
                    }
                    if ui.button("Export as Aetos").clicked() {
                        let code = self.generate_code();
                        if let Err(e) = std::fs::write("generated.aetos", &code) {
                            eprintln!("Failed to save: {}", e);
                        } else {
                            println!("Code saved to generated.aetos");
                        }
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        // TODO: Implement undo
                    }
                    if ui.button("Redo").clicked() {
                        // TODO: Implement redo
                    }
                    ui.separator();
                    if ui.button("Select All").clicked() {
                        // TODO: Implement select all
                    }
                    if ui.button("Clear Selection").clicked() {
                        self.selected_node = None;
                        self.selected_connection = None;
                    }
                    if ui.button("Delete Selected").clicked() {
                        if let Some(node_id) = self.selected_node {
                            self.delete_node(node_id);
                        } else if let Some(conn_id) = self.selected_connection {
                            self.delete_connection(conn_id);
                        }
                    }
                });
                
                ui.menu_button("Add Node", |ui| {
                    ui.label("Basic Nodes:");
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
                    ui.checkbox(&mut self.show_properties, "Properties Panel");
                    ui.checkbox(&mut self.show_code_window, "Code Window");
                    ui.checkbox(&mut self.show_info_window, "Info Window");
                    ui.separator();
                    if ui.button("Zoom In").clicked() {
                        self.zoom *= 1.2;
                    }
                    if ui.button("Zoom Out").clicked() {
                        self.zoom *= 0.8;
                    }
                    if ui.button("Reset View").clicked() {
                        self.pan = (0.0, 0.0);
                        self.zoom = 1.0;
                    }
                });
                
                ui.menu_button("Tools", |ui| {
                    if ui.button("Validate Connections").clicked() {
                        let errors = self.validate_connections();
                        if errors.is_empty() {
                            println!("✓ All connections are valid");
                        } else {
                            println!("Validation Errors:");
                            for error in errors {
                                println!("  • {}", error);
                            }
                        }
                    }
                    if ui.button("Auto Layout").clicked() {
                        self.auto_layout();
                    }
                    if ui.button("Find Cycles").clicked() {
                        let cycles = self.find_cycles();
                        if cycles.is_empty() {
                            println!("✓ No cycles found");
                        } else {
                            println!("Cycles found:");
                            for cycle in cycles {
                                println!("  • {:?}", cycle);
                            }
                        }
                    }
                    if ui.button("Clear All Connections").clicked() {
                        self.connections.clear();
                    }
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Nodes: {} | Connections: {}", self.nodes.len(), self.connections.len()));
                });
            });
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            let rect = ui.available_rect_before_wrap();
            
            painter.rect_filled(rect, 0.0, egui::Color32::from_gray(30));
            
            if ui.input(|i| i.pointer.middle_down()) {
                self.pan.0 += ui.input(|i| i.pointer.delta().x);
                self.pan.1 += ui.input(|i| i.pointer.delta().y);
            }
            
            if ui.input(|i| i.zoom_delta() != 1.0) {
                self.zoom *= ui.input(|i| i.zoom_delta());
                self.zoom = self.zoom.clamp(0.1, 5.0);
            }
            
            if ui.input(|i| i.pointer.secondary_clicked()) {
                if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                    self.context_menu_pos = (pointer_pos.x, pointer_pos.y);
                    self.show_context_menu = true;
                }
            }
            
            if ui.input(|i| i.pointer.primary_clicked()) && self.show_context_menu {
                self.show_context_menu = false;
            }
            
            if let Some((_node_id, _port_id, ref mut start_pos)) = &mut self.dragging_connection_start {
                if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos()) {
                    *start_pos = mouse_pos;
                }
                
                if ui.input(|i| i.pointer.secondary_clicked()) {
                    self.dragging_connection_start = None;
                }
            }
            
            self.draw_connections(&painter, rect);
            
            let mut dragged_node_id = None;
            let mut drag_delta = (0.0, 0.0);
            let mut clicked_node_id = None;
            let mut clicked_on_port = false;
            let mouse_pos = ui.input(|i| i.pointer.interact_pos());
            
            let nodes_copy = self.nodes.clone();
            
            for node in &nodes_copy {
                let pos = egui::pos2(
                    node.position.0 * self.zoom + self.pan.0 + rect.center().x,
                    node.position.1 * self.zoom + self.pan.1 + rect.center().y,
                );
                
                let size = egui::vec2(node.size.0 * self.zoom, node.size.1 * self.zoom);
                let node_rect = egui::Rect::from_min_size(pos, size);
                
                let bg_color = if self.selected_node == Some(node.id) {
                    egui::Color32::from_rgb(80, 80, 120)
                } else {
                    egui::Color32::from_rgb(60, 60, 80)
                };
                
                painter.rect_filled(node_rect, 10.0 * self.zoom, bg_color);
                
                painter.rect_stroke(
                    node_rect,
                    10.0 * self.zoom,
                    egui::Stroke::new(2.0 * self.zoom, egui::Color32::from_gray(100)),
                );
                
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
                
                if let Some(name) = node.properties.get("name") {
                    painter.text(
                        pos + egui::vec2(10.0 * self.zoom, 40.0 * self.zoom),
                        egui::Align2::LEFT_TOP,
                        name,
                        egui::FontId::proportional(12.0 * self.zoom),
                        egui::Color32::from_gray(200),
                    );
                }
                
                self.draw_ports(&painter, node, node_rect);
                
                let response = ui.interact(node_rect, egui::Id::new(node.id), egui::Sense::drag());
                
                if response.dragged() {
                    dragged_node_id = Some(node.id);
                    drag_delta = (
                        ui.input(|i| i.pointer.delta().x) / self.zoom,
                        ui.input(|i| i.pointer.delta().y) / self.zoom,
                    );
                }
                
                if response.clicked() {
                    if let Some(mouse_pos) = mouse_pos {
                        let mut clicked = self.check_port_click(node, node_rect, mouse_pos);
                        if !clicked {
                            clicked_node_id = Some(node.id);
                        } else {
                            clicked_on_port = true;
                        }
                    }
                }
            }
            
            if let Some(node_id) = dragged_node_id {
                if let Some(node_mut) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                    node_mut.position.0 += drag_delta.0;
                    node_mut.position.1 += drag_delta.1;
                }
            }
            
            if let Some(node_id) = clicked_node_id {
                if !clicked_on_port {
                    self.selected_node = Some(node_id);
                    self.show_properties = true;
                    self.selected_connection = None;
                }
            }
            
            if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
                if let Some(connection_id) = self.selected_connection {
                    self.delete_connection(connection_id);
                } else if let Some(node_id) = self.selected_node {
                    self.delete_node(node_id);
                }
            }
            
            if self.show_context_menu {
                let menu_response = egui::Area::new(egui::Id::new("context_menu_area"))
                    .fixed_pos(egui::pos2(self.context_menu_pos.0, self.context_menu_pos.1))
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        egui::Frame::popup(ui.style()).show(ui, |ui| {
                            ui.set_min_width(150.0);
                            
                            let world_pos = (
                                (self.context_menu_pos.0 - self.pan.0 - rect.center().x) / self.zoom,
                                (self.context_menu_pos.1 - self.pan.1 - rect.center().y) / self.zoom,
                            );
                            
                            ui.heading("Add Node");
                            ui.separator();
                            
                            if ui.button("📊 Variable").clicked() {
                                self.add_node(NodeType::Variable, world_pos.0, world_pos.1);
                                self.show_context_menu = false;
                            }
                            if ui.button("⚙️  Operation").clicked() {
                                self.add_node(NodeType::Operation, world_pos.0, world_pos.1);
                                self.show_context_menu = false;
                            }
                            if ui.button("🔢 Literal").clicked() {
                                self.add_node(NodeType::Literal, world_pos.0, world_pos.1);
                                self.show_context_menu = false;
                            }
                            if ui.button("🖨️  Print").clicked() {
                                self.add_node(NodeType::Print, world_pos.0, world_pos.1);
                                self.show_context_menu = false;
                            }
                            if ui.button("📐 Function").clicked() {
                                self.add_node(NodeType::Function, world_pos.0, world_pos.1);
                                self.show_context_menu = false;
                            }
                        });
                    });
                
                if !menu_response.response.contains_pointer() && ui.input(|i| i.pointer.primary_clicked()) {
                    self.show_context_menu = false;
                }
            }
        });
        
        if self.show_properties {
            let selected_id = self.selected_node;
            if let Some(selected_id) = selected_id {
                let node_data = {
                    let node = self.nodes.iter().find(|n| n.id == selected_id);
                    node.cloned()
                };
                
                if let Some(mut node) = node_data {
                    let mut should_delete = false;
                    let mut should_duplicate = false;
                    
                    let window_response = egui::Window::new("Node Properties")
                        .default_size((300.0, 250.0))
                        .collapsible(true)
                        .show(ctx, |ui| {
                            ui.label(format!("Node ID: {}", node.id));
                            ui.label(format!("Type: {:?}", node.node_type));
                            ui.separator();
                            
                            match node.node_type {
                                NodeType::Variable => {
                                    ui.horizontal(|ui| {
                                        ui.label("Name:");
                                        let mut name = node.properties.get("name").cloned().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut name).changed() {
                                            node.properties.insert("name".to_string(), name);
                                        }
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("Type:");
                                        let mut type_str = node.properties.get("type").cloned().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut type_str).changed() {
                                            node.properties.insert("type".to_string(), type_str);
                                        }
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("Value:");
                                        let mut value = node.properties.get("value").cloned().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut value).changed() {
                                            node.properties.insert("value".to_string(), value);
                                        }
                                    });
                                }
                                NodeType::Operation => {
                                    ui.horizontal(|ui| {
                                        ui.label("Operator:");
                                        let mut op = node.properties.get("operator").cloned().unwrap_or_default();
                                        egui::ComboBox::from_label("")
                                            .selected_text(&op)
                                            .show_ui(ui, |ui| {
                                                for operator in ["+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||"] {
                                                    ui.selectable_value(&mut op, operator.to_string(), operator);
                                                }
                                            });
                                        if op != node.properties.get("operator").cloned().unwrap_or_default() {
                                            node.properties.insert("operator".to_string(), op);
                                        }
                                    });
                                }
                                NodeType::Literal => {
                                    ui.horizontal(|ui| {
                                        ui.label("Value:");
                                        let mut value = node.properties.get("value").cloned().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut value).changed() {
                                            node.properties.insert("value".to_string(), value);
                                        }
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("Type:");
                                        let mut type_str = node.properties.get("type").cloned().unwrap_or_default();
                                        egui::ComboBox::from_label("")
                                            .selected_text(&type_str)
                                            .show_ui(ui, |ui| {
                                                for t in ["i32", "f32", "bool", "string"] {
                                                    ui.selectable_value(&mut type_str, t.to_string(), t);
                                                }
                                            });
                                        if type_str != node.properties.get("type").cloned().unwrap_or_default() {
                                            node.properties.insert("type".to_string(), type_str);
                                        }
                                    });
                                }
                                NodeType::Function => {
                                    ui.horizontal(|ui| {
                                        ui.label("Name:");
                                        let mut name = node.properties.get("name").cloned().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut name).changed() {
                                            node.properties.insert("name".to_string(), name);
                                        }
                                    });
                                }
                                _ => {}
                            }
                            
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                                ui.label(format!("({:.1}, {:.1})", node.position.0, node.position.1));
                            });
                            
                            ui.horizontal(|ui| {
                                if ui.button("Delete Node").clicked() {
                                    should_delete = true;
                                }
                                
                                if ui.button("Duplicate").clicked() {
                                    should_duplicate = true;
                                }
                            });
                        });
                    
                    // Применяем изменения после закрытия окна
                    if window_response.is_some() {
                        if let Some(node_mut) = self.nodes.iter_mut().find(|n| n.id == selected_id) {
                            *node_mut = node;
                        }
                        
                        if should_delete {
                            self.delete_node(selected_id);
                        }
                        
                        if should_duplicate {
                            self.duplicate_node(selected_id);
                        }
                    }
                }
            }
        }
        
        if let Some(selected_connection_id) = self.selected_connection {
            let connection_data = self.connections.iter()
                .find(|c| c.id == selected_connection_id)
                .cloned();
            
            if let Some(connection) = connection_data {
                let mut should_delete = false;
                
                egui::Window::new("Connection Properties")
                    .default_size((300.0, 150.0))
                    .collapsible(true)
                    .show(ctx, |ui| {
                        ui.label(format!("Connection ID: {}", connection.id));
                        ui.separator();
                        
                        if let (Some(from_node), Some(to_node)) = (
                            self.nodes.iter().find(|n| n.id == connection.from_node),
                            self.nodes.iter().find(|n| n.id == connection.to_node),
                        ) {
                            ui.label(format!("From: {} ({})", 
                                from_node.properties.get("name").unwrap_or(&format!("Node {}", from_node.id)),
                                connection.from_port
                            ));
                            
                            ui.label(format!("To: {} ({})", 
                                to_node.properties.get("name").unwrap_or(&format!("Node {}", to_node.id)),
                                connection.to_port
                            ));
                        }
                        
                        if ui.button("Delete Connection").clicked() {
                            should_delete = true;
                        }
                    });
                
                if should_delete {
                    self.delete_connection(selected_connection_id);
                    self.selected_connection = None;
                }
            }
        }
        
        if self.show_code_window {
            let code = self.generate_code();
            egui::Window::new("Generated Code")
                .default_size((400.0, 300.0))
                .collapsible(true)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.code(&code);
                    });
                    
                    ui.horizontal(|ui| {
                        if ui.button("📋 Copy").clicked() {
                            ui.ctx().copy_text(code.clone());
                        }
                        
                        if ui.button("💾 Save").clicked() {
                            if let Err(e) = std::fs::write("generated.aetos", &code) {
                                eprintln!("Failed to save: {}", e);
                            }
                        }
                    });
                });
        }
        
        if self.show_info_window {
            egui::Window::new("Editor Info")
                .default_size((250.0, 180.0))
                .collapsible(true)
                .show(ctx, |ui| {
                    ui.label(format!("📊 Nodes: {}", self.nodes.len()));
                    ui.label(format!("🔗 Connections: {}", self.connections.len()));
                    ui.label(format!("🔍 Zoom: {:.2}x", self.zoom));
                    ui.label(format!("📍 Pan: ({:.1}, {:.1})", self.pan.0, self.pan.1));
                    ui.separator();
                    ui.label("🎮 Controls:");
                    ui.label("• 🖱️  Middle drag: Pan");
                    ui.label("• 🔍 Wheel: Zoom");
                    ui.label("• 🖱️  Right click: Add node");
                    ui.label("• 🔗 Drag from port: Create connection");
                    ui.label("• 🗑️  Delete: Remove selected");
                });
        }
        
        if self.save_dialog_open {
            let mut file_path = self.file_path.clone();
            let mut should_close = false;
            
            egui::Window::new("Save Project")
                .open(&mut self.save_dialog_open)
                .show(ctx, |ui| {
                    ui.label("File path:");
                    ui.text_edit_singleline(&mut file_path);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            match self.export_project() {
                                Ok(json) => {
                                    if let Err(e) = std::fs::write(&file_path, json) {
                                        eprintln!("Failed to save: {}", e);
                                    } else {
                                        should_close = true;
                                    }
                                }
                                Err(e) => eprintln!("Failed to export: {}", e),
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }
                    });
                });
            
            if should_close {
                self.save_dialog_open = false;
                self.file_path = file_path;
            }
        }
        
        if self.load_dialog_open {
            let mut file_path = self.file_path.clone();
            let mut should_close = false;
            
            egui::Window::new("Load Project")
                .open(&mut self.load_dialog_open)
                .show(ctx, |ui| {
                    ui.label("File path:");
                    ui.text_edit_singleline(&mut file_path);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Load").clicked() {
                            match std::fs::read_to_string(&file_path) {
                                Ok(content) => {
                                    if let Err(e) = self.import_project(&content) {
                                        eprintln!("Failed to load: {}", e);
                                    } else {
                                        should_close = true;
                                    }
                                }
                                Err(e) => eprintln!("Failed to read file: {}", e),
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }
                    });
                });
            
            if should_close {
                self.load_dialog_open = false;
                self.file_path = file_path;
            }
        }
        
        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Aetos Visual Editor v1.0"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Aetos Visual Editor",
        options,
        Box::new(|_cc| Box::new(VisualEditor::default())),
    )
}