use crate::ast::*;

pub fn get_stdlib() -> Program {
    Program {
        structs: vec![
            // Графические структуры
            Struct {
                name: "Point".to_string(),
                fields: vec![
                    StructField {
                        name: "x".to_string(),
                        field_type: Type::I32,
                    },
                    StructField {
                        name: "y".to_string(),
                        field_type: Type::I32,
                    },
                ],
            },
            Struct {
                name: "Color".to_string(),
                fields: vec![
                    StructField {
                        name: "r".to_string(),
                        field_type: Type::I32,
                    },
                    StructField {
                        name: "g".to_string(),
                        field_type: Type::I32,
                    },
                    StructField {
                        name: "b".to_string(),
                        field_type: Type::I32,
                    },
                ],
            },
            Struct {
                name: "Rect".to_string(),
                fields: vec![
                    StructField {
                        name: "x".to_string(),
                        field_type: Type::I32,
                    },
                    StructField {
                        name: "y".to_string(),
                        field_type: Type::I32,
                    },
                    StructField {
                        name: "width".to_string(),
                        field_type: Type::I32,
                    },
                    StructField {
                        name: "height".to_string(),
                        field_type: Type::I32,
                    },
                ],
            },
        ],
        functions: vec![
            // Базовые функции вывода
            Function {
                name: "print".to_string(),
                params: vec![
                    Parameter {
                        name: "value".to_string(),
                        param_type: Type::I32,
                    }
                ],
                return_type: Type::Void,
                body: vec![],
            },
            
            // Графические функции
            Function {
                name: "init_graphics".to_string(),
                params: vec![
                    Parameter {
                        name: "width".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "height".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "title".to_string(),
                        param_type: Type::String,
                    },
                ],
                return_type: Type::Void,
                body: vec![],
            },
            
            Function {
                name: "clear_screen".to_string(),
                params: vec![
                    Parameter {
                        name: "color".to_string(),
                        param_type: Type::Struct("Color".to_string()),
                    }
                ],
                return_type: Type::Void,
                body: vec![],
            },
            
            Function {
                name: "draw_pixel".to_string(),
                params: vec![
                    Parameter {
                        name: "x".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "y".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "color".to_string(),
                        param_type: Type::Struct("Color".to_string()),
                    }
                ],
                return_type: Type::Void,
                body: vec![],
            },
            
            Function {
                name: "draw_rect".to_string(),
                params: vec![
                    Parameter {
                        name: "rect".to_string(),
                        param_type: Type::Struct("Rect".to_string()),
                    },
                    Parameter {
                        name: "color".to_string(),
                        param_type: Type::Struct("Color".to_string()),
                    }
                ],
                return_type: Type::Void,
                body: vec![],
            },
            
            Function {
                name: "draw_circle".to_string(),
                params: vec![
                    Parameter {
                        name: "center_x".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "center_y".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "radius".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "color".to_string(),
                        param_type: Type::Struct("Color".to_string()),
                    }
                ],
                return_type: Type::Void,
                body: vec![],
            },
            
            Function {
                name: "draw_line".to_string(),
                params: vec![
                    Parameter {
                        name: "x1".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "y1".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "x2".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "y2".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "color".to_string(),
                        param_type: Type::Struct("Color".to_string()),
                    }
                ],
                return_type: Type::Void,
                body: vec![],
            },
            
            Function {
                name: "render".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            },
            
            Function {
                name: "get_time".to_string(),
                params: vec![],
                return_type: Type::F64,
                body: vec![],
            },
            
            Function {
                name: "is_key_pressed".to_string(),
                params: vec![
                    Parameter {
                        name: "key".to_string(),
                        param_type: Type::I32,
                    }
                ],
                return_type: Type::Bool,
                body: vec![],
            },
            
            Function {
                name: "get_mouse_pos".to_string(),
                params: vec![],
                return_type: Type::Struct("Point".to_string()),
                body: vec![],
            },
            
            // Утилиты
            Function {
                name: "rgb".to_string(),
                params: vec![
                    Parameter {
                        name: "r".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "g".to_string(),
                        param_type: Type::I32,
                    },
                    Parameter {
                        name: "b".to_string(),
                        param_type: Type::I32,
                    },
                ],
                return_type: Type::Struct("Color".to_string()),
                body: vec![
                    Statement::Return {
                        value: Expression::StructInitialization {
                            struct_name: "Color".to_string(),
                            fields: vec![
                                ("r".to_string(), Expression::Variable("r".to_string())),
                                ("g".to_string(), Expression::Variable("g".to_string())),
                                ("b".to_string(), Expression::Variable("b".to_string())),
                            ],
                        },
                    }
                ],
            },
            
            // Математические функции
            Function {
                name: "sin".to_string(),
                params: vec![
                    Parameter {
                        name: "angle".to_string(),
                        param_type: Type::F32,
                    }
                ],
                return_type: Type::F32,
                body: vec![],
            },
            
            Function {
                name: "cos".to_string(),
                params: vec![
                    Parameter {
                        name: "angle".to_string(),
                        param_type: Type::F32,
                    }
                ],
                return_type: Type::F32,
                body: vec![],
            },
            
            // Функция задержки
            Function {
                name: "sleep".to_string(),
                params: vec![
                    Parameter {
                        name: "ms".to_string(),
                        param_type: Type::I32,
                    }
                ],
                return_type: Type::Void,
                body: vec![],
            },
        ],
    }
}