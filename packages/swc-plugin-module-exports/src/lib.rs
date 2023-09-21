#![allow(clippy::not_unsafe_ptr_arg_deref)]

use swc_atoms::{JsWord, Atom};
use swc_core::common::DUMMY_SP;
use swc_core::plugin::proxies::TransformPluginProgramMetadata;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use swc_ecma_ast::*;
use swc_plugin_macro::plugin_transform;

const DEFAULT_MODULE_EXPORT_ALL_STRUCT: ModuleExportAll = ModuleExportAll {
    has_writed_name_exports_module: false,
    has_writed_all_module: false,
    contains_es_module_definition: false,
    contains_use_strict_declaration: false,
    class_default_number: 0,
    function_default_number: 0
};

#[plugin_transform]
pub fn module_exports_all(mut program: Program, _: TransformPluginProgramMetadata) -> Program {
    program.visit_mut_with(&mut ModuleExportAll::from(DEFAULT_MODULE_EXPORT_ALL_STRUCT));

    program
}

struct ModuleExportAll {
    has_writed_name_exports_module: bool,
    has_writed_all_module: bool,
    contains_use_strict_declaration: bool,
    contains_es_module_definition: bool,
    class_default_number: i32,
    function_default_number: i32,
}

impl VisitMut for ModuleExportAll {
    fn visit_mut_module(&mut self, module: &mut Module) {
        self.visit_mut_program_module(module);
    }
}

fn create_expression_statement_as_stmt(expr: Box<Expr>) -> Stmt {
    Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr
    })
}

fn create_js_string_as_box_expr(js_word: &str) -> Box<Expr> {
    let symbol = "\"";

    Box::new(Expr::Lit(Lit::Str(Str {
        span: DUMMY_SP,
        value: JsWord::from(js_word),
        raw: Some(Atom::new(format!("{}{}{}", symbol, js_word, symbol)))
    })))
}

fn create_js_object_as_box_expr(props: Vec<PropOrSpread>) -> Box<Expr> {
    Box::new(Expr::Object(ObjectLit {
        span: DUMMY_SP,
        props
    }))
}

fn create_literal_prop(
    prop_name: &str,
    prop_name_optional: bool,
    literal_value: Lit
) -> PropOrSpread {
    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
        key: create_prop_name(
            prop_name,
            prop_name_optional
        ),
        value: create_literal_prop_value(literal_value)
    })))
}

fn create_prop(
    prop_name: &str,
    prop_name_optional: bool,
    value: Expr
) -> PropOrSpread {
    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
        key: create_prop_name(
            prop_name,
            prop_name_optional
        ),
        value: Box::new(value)
    })))
}

fn create_literal_prop_value(literal_value: Lit) -> Box<Expr> {
    Box::new(Expr::Lit(literal_value))
}

fn create_prop_name(prop_name: &str, optional: bool) -> PropName {
    PropName::Ident(Ident {
        span: DUMMY_SP,
        sym: JsWord::from(prop_name),
        optional
    })
}

fn create_js_string_as_stmt(js_word: &str) -> Stmt {
    let symbol = "\"";

    create_expression_statement_as_stmt(Lit::into(Lit::Str(Str {
        span: DUMMY_SP,
        value: JsWord::from(js_word),
        raw: Some(Atom::new(format!("{}{}{}", symbol, js_word, symbol)))
    })))
}

fn create_call_expression(
    callee: Callee,
    args: Vec<ExprOrSpread>,
    type_args: Option<Box<TsTypeParamInstantiation>>
) -> Expr {
    Expr::Call(CallExpr {
        span: DUMMY_SP,
        args,
        callee,
        type_args
    })
}

fn create_member_property(name: &str, optional: bool) -> MemberProp {
    MemberProp::Ident(Ident {
        span: DUMMY_SP,
        optional,
        sym: JsWord::from(name)
    })
}

fn create_argument_identifier(name: &str, optional: bool) -> Box<Expr> {
    Box::new(Expr::Ident(Ident {
        span: DUMMY_SP,
        sym: JsWord::from(name),
        optional
    }))
}

fn create_argument_expr_or_spread(expr: Box<Expr>, is_spread: bool) -> ExprOrSpread {
    let use_spread;

    if is_spread {
        use_spread = Some(DUMMY_SP)
    } else {
        use_spread = None
    }

    ExprOrSpread {
        spread: use_spread,
        expr
    }
}

fn create_callee_member_expression(
    object: Expr,
    property: &str
) -> Callee {
    Callee::Expr(Box::new(Expr::from(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(object),
        prop: create_member_property(
            property,
            false
        )
    })))
}

fn create_member_expression(
    object: Expr,
    property: &str
) -> MemberExpr {
    MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(object),
        prop: create_member_property(
            property,
            false
        )
    }
}

fn create_member_expression_by_key(
    object: Expr,
    property: MemberProp
) -> MemberExpr {
    MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(object),
        prop: property
    }
}

fn create_literal_boolean(bool_value: bool) -> Lit {
    Lit::Bool(Bool {
        span: DUMMY_SP,
        value: bool_value
    })
}

fn create_literal_string(string_value: &str) -> Lit {
    let symbol = "\"";

    Lit::Str(Str {
        span: DUMMY_SP,
        value: JsWord::from(string_value),
        raw: Some(Atom::new(format!("{}{}{}", symbol, string_value, symbol)))
    })
}

fn create_identifier(identifier_name: &str) -> Ident {
    Ident::new(
        JsWord::from(identifier_name),
        DUMMY_SP
    )
}

fn create_function(
    params: Vec<Param>,
    decorators: Vec<Decorator>,
    body: Option<BlockStmt>,
    is_generator: bool,
    is_async: bool,
    type_params: Option<Box<TsTypeParamDecl>>,
    return_type: Option<Box<TsTypeAnn>>
) -> Function {
    Function {
        params,
        decorators,
        span: DUMMY_SP,
        body,
        is_generator,
        is_async,
        type_params,
        return_type
    }
}

fn create_pat_ident(
    id: Ident,
    type_ann: Option<Box<TsTypeAnn>>
) -> Pat {
    Pat::Ident(BindingIdent {
        id,
        type_ann
    })
}

fn create_param(
    param_name: &str
) -> Param {
    Param {
        span: DUMMY_SP,
        decorators: vec![],
        pat: create_pat_ident(
            create_identifier(param_name),
            None
        )
    }
}

fn create_block_statement(statements: Vec<Stmt>) -> BlockStmt {
    BlockStmt {
        span: DUMMY_SP,
        stmts: statements
    }
}

fn create_fn_declaration(
    func_name: &str,
    declare: bool,
    function: Function
) -> Stmt {
    Stmt::Decl(Decl::Fn(FnDecl {
        ident: create_identifier(func_name),
        declare,
        function: Box::new(function)
    }))
}

fn create_fn_expression(
    func_name: Option<&str>,
    function: Function
) -> Expr {
    Expr::Fn(FnExpr {
        ident: match func_name {
            Some(name) => Some(create_identifier(name)),
            None => None
        },
        function: Box::new(function)
    })
}

fn create_if_statement(
    test: Expr,
    cons: Stmt,
    alt: Option<Box<Stmt>>
) -> Stmt {
    Stmt::If(IfStmt {
        span: DUMMY_SP,
        test: Box::new(test),
        cons: Box::new(cons),
        alt
    })
}

fn create_return_statement(
    arg: Option<Expr>
) -> ReturnStmt {
    ReturnStmt {
        span: DUMMY_SP,
        arg: match arg {
            Some(some_arg) => Some(Box::new(some_arg)),
            None => None
        }
    }
}

fn create_bin_expression(
    op: BinaryOp,
    left: Box<Expr>,
    right: Box<Expr>
) -> Expr {
    Expr::Bin(BinExpr {
        span: DUMMY_SP,
        op,
        left,
        right
    })
}

fn create_unary_expression(
    op: UnaryOp,
    arg: Expr
) -> Expr {
    Expr::Unary(UnaryExpr {
        span: DUMMY_SP,
        op,
        arg: Box::new(arg)
    })
}

fn create_assignment_expression(
    op: AssignOp,
    left: PatOrExpr,
    right: Box<Expr>
) -> Expr {
    Expr::Assign(AssignExpr {
        span: DUMMY_SP,
        op,
        left,
        right
    })
}

fn define_es_module_property(value: bool) -> Stmt {
    create_expression_statement_as_stmt(
        Box::new(create_call_expression(
            create_callee_member_expression(
                Expr::from(create_identifier("Object")),
                "defineProperty"
            ),
            vec![
            create_argument_expr_or_spread(
                create_argument_identifier("exports", false),
                false
            ),
            create_argument_expr_or_spread(
                create_js_string_as_box_expr("__esModule"),
                false
            ),
            ExprOrSpread::from(create_js_object_as_box_expr(
                vec![
                create_literal_prop(
                    "value",
                    false,
                    create_literal_boolean(value)
                )
                ]
            ))
            ],
            None
        ))
    )
}

fn define_export_assignment_by_identfier(
    prop_name: &str,
    prop_value: &str
) -> Stmt {
    create_expression_statement_as_stmt(
        Box::new(create_assignment_expression(
            AssignOp::Assign,
            PatOrExpr::Expr(Box::new(Expr::from(create_member_expression(
                Expr::from(create_identifier("exports")),
                &prop_name
            )))),
            Box::new(Expr::from(create_identifier(&prop_value)))
        ))
    )
}

fn define_export_assignment_by_literal_value(
    prop_name: &str,
    prop_value: Box<Expr>
) -> Stmt {
    create_expression_statement_as_stmt(
        Box::new(create_assignment_expression(
            AssignOp::Assign,
            PatOrExpr::Expr(Box::new(Expr::from(create_member_expression(
                Expr::from(create_identifier("exports")),
                &prop_name
            )))),
            prop_value
        ))
    )
}

impl ModuleExportAll {
    fn visit_mut_program_module(&mut self, module: &mut Module) {
        let old_has_writed_all_module = self.has_writed_all_module;
        let old_has_writed_name_exports_module = self.has_writed_name_exports_module;

        let mut updated_body = Vec::new();

        for node in &mut *module.body {
            match node {
                ModuleItem::ModuleDecl(var) => {

                    if var.is_export_all() {

                        let module_name_or_module_path = var.as_export_all().unwrap().src.value.to_string();

                        //Attention, from now on the code will not be documented
                        //There's no going back from now on
                        //I wish you luck

                        let __export_star = create_expression_statement_as_stmt(
                            Box::new(Expr::Call(CallExpr {
                                span: DUMMY_SP,
                                callee: Callee::Expr(Box::new(Expr::from(create_identifier("__exportStar")))),
                                args: vec![
                                create_argument_expr_or_spread(
                                    Box::new(Expr::Call(CallExpr {
                                        span: DUMMY_SP,
                                        callee: Callee::Expr(Box::new(Expr::from(create_identifier("require")))),
                                        args: vec![
                                        create_argument_expr_or_spread(
                                            Box::new(Expr::from(create_literal_string(&module_name_or_module_path))
                                        ), false)
                                        ],
                                        type_args: None
                                    })),
                                    false
                                ),
                                create_argument_expr_or_spread(
                                    Box::new(Expr::from(create_identifier("exports"))),
                                    false
                                )
                                ],
                                type_args: None
                            }))
                        );

                        if self.has_writed_all_module == false {
                            if self.contains_use_strict_declaration == false {
                                let use_strict = create_js_string_as_stmt("use strict");
                                updated_body.push(ModuleItem::from(use_strict));
                                self.contains_use_strict_declaration = true;
                            }

                            if self.contains_es_module_definition == false {
                                let define_es_module_value = define_es_module_property(true);
                                updated_body.push(ModuleItem::from(define_es_module_value));
                                self.contains_es_module_definition = true;
                            }

                            //I'm so sorry if you are going to read this code.

                            let define_export_star_func = create_fn_declaration(
                                "__exportStar",
                                false,
                                create_function(
                                    vec![
                                    create_param(
                                        "from"
                                    ),
                                    create_param(
                                        "to"
                                    ),
                                    ],
                                    vec![],
                                    Some(create_block_statement(
                                        vec![
                                        create_expression_statement_as_stmt(
                                            Box::new(create_call_expression(
                                                create_callee_member_expression(
                                                    create_call_expression(
                                                        create_callee_member_expression(
                                                            Expr::from(create_identifier("Object")),
                                                            "keys"
                                                        ),
                                                        vec![
                                                        create_argument_expr_or_spread(
                                                            create_argument_identifier(
                                                                "from",
                                                                false
                                                            ),
                                                            false
                                                        )
                                                        ],
                                                        None
                                                    ),
                                                    "forEach"
                                                ),
                                                vec![
                                                create_argument_expr_or_spread(
                                                    Box::new(create_fn_expression(
                                                        None,
                                                        create_function(
                                                            vec![
                                                            create_param("k")
                                                            ],
                                                            vec![],
                                                            Some(create_block_statement(vec![
                                                                create_if_statement(
                                                                    create_bin_expression(
                                                                        BinaryOp::LogicalAnd,
                                                                        Box::new(create_bin_expression(
                                                                            BinaryOp::NotEqEq,
                                                                            Box::new(Expr::from(create_identifier("k"))),
                                                                            Box::new(Expr::from(create_literal_string("default")))
                                                                        )),
                                                                        Box::new(create_unary_expression(
                                                                            UnaryOp::Bang,
                                                                            create_call_expression(
                                                                                create_callee_member_expression(
                                                                                    Expr::from(create_member_expression(
                                                                                        Expr::from(create_member_expression(
                                                                                            Expr::from(create_identifier("Object")),
                                                                                            "prototype"
                                                                                        )),
                                                                                        "hasOwnProperty"
                                                                                    )),
                                                                                    "call"
                                                                                ),
                                                                                vec![
                                                                                create_argument_expr_or_spread(
                                                                                    Box::new(Expr::from(create_identifier("to"))),
                                                                                    false
                                                                                ),
                                                                                create_argument_expr_or_spread(
                                                                                    Box::new(Expr::from(create_identifier("k"))),
                                                                                    false
                                                                                )
                                                                                ],
                                                                                None
                                                                            )
                                                                        ))
                                                                    ),
                                                                    Stmt::from(create_block_statement(
                                                                        vec![
                                                                        create_expression_statement_as_stmt(
                                                                            Box::new(create_call_expression(
                                                                                create_callee_member_expression(
                                                                                    Expr::from(create_identifier("Object")),
                                                                                    "defineProperty"
                                                                                ),
                                                                                vec![
                                                                                create_argument_expr_or_spread(
                                                                                    Box::new(Expr::from(create_identifier("to"))),
                                                                                    false
                                                                                ),
                                                                                create_argument_expr_or_spread(
                                                                                    Box::new(Expr::from(create_identifier("k"))),
                                                                                    false
                                                                                ),
                                                                                ExprOrSpread::from(create_js_object_as_box_expr(
                                                                                    vec![
                                                                                    create_literal_prop(
                                                                                        "enumerable",
                                                                                        false,
                                                                                        create_literal_boolean(true)
                                                                                    ),
                                                                                    create_prop(
                                                                                        "get",
                                                                                        false,
                                                                                        create_fn_expression(
                                                                                            None,
                                                                                            create_function(
                                                                                                vec![],
                                                                                                vec![],
                                                                                                Some(create_block_statement(vec![
                                                                                                    Stmt::from(create_return_statement(Some(
                                                                                                        Expr::from(create_member_expression_by_key(
                                                                                                            Expr::from(create_identifier("from")),
                                                                                                            MemberProp::Computed(ComputedPropName {
                                                                                                                span: DUMMY_SP,
                                                                                                                expr: Box::new(Expr::from(create_identifier("k")))
                                                                                                            })
                                                                                                        ))
                                                                                                    )))
                                                                                                    ])),
                                                                                                    false,
                                                                                                    false,
                                                                                                    None,
                                                                                                    None
                                                                                                )
                                                                                            )
                                                                                        )
                                                                                        ]
                                                                                    ))
                                                                                    ],
                                                                                    None
                                                                                ))
                                                                            )
                                                                            ]
                                                                        )),
                                                                        None
                                                                    ),
                                                                    ],
                                                                )),
                                                                false,
                                                                false,
                                                                None,
                                                                None
                                                            )
                                                        )),
                                                        false
                                                    )
                                                    ],
                                                    None
                                                ))
                                            ),

                                            Stmt::from(create_return_statement(
                                                Some(Expr::from(create_identifier("from")))
                                            ))
                                            ]
                                        )),
                                        false,
                                        false,
                                        None,
                                        None
                                    )
                                );

                                updated_body.push(ModuleItem::from(define_export_star_func));

                                self.has_writed_all_module = true;
                            }

                            updated_body.push(ModuleItem::from(__export_star));

                        }
                        if var.is_export_decl() {

                            if self.has_writed_name_exports_module == false {
                                if self.contains_use_strict_declaration == false {
                                    let use_strict = create_js_string_as_stmt("use strict");
                                    updated_body.push(ModuleItem::from(use_strict));
                                    self.contains_use_strict_declaration = true;
                                }

                                if self.contains_es_module_definition == false {
                                    let define_es_module_value = define_es_module_property(true);
                                    updated_body.push(ModuleItem::from(define_es_module_value));
                                    self.contains_es_module_definition = true;
                                }

                                self.has_writed_name_exports_module = true;
                            }

                            let node_declaration = &var.as_export_decl().unwrap().decl;

                            let export_names_or_nothing = match node_declaration.clone() {
                                Decl::Class(class_node) => Some(vec![class_node.ident.sym]),
                                Decl::Fn(function_node) => Some(vec![function_node.ident.sym]),
                                Decl::Var(var_node) => {
                                    let mut names = Vec::new();

                                    for declaration in var_node.decls {
                                        names.push(declaration.name.as_ident().unwrap().id.sym.clone())
                                    }

                                    Some(names)
                                },
                                _ => None
                            };

                            match export_names_or_nothing {
                                Some(export_names) => {
                                    for export_name in export_names {
                                        let void_0_statement = create_expression_statement_as_stmt(
                                            Box::new(create_assignment_expression(
                                                AssignOp::Assign,
                                                PatOrExpr::Expr(Box::new(Expr::from(create_member_expression(
                                                    Expr::from(create_identifier("exports")),
                                                    &export_name
                                                )))),
                                                Box::new(create_unary_expression(
                                                    UnaryOp::Void,
                                                    Expr::Lit(Lit::Num(Number {
                                                        span: DUMMY_SP,
                                                        value: f64::from(0),
                                                        raw: Some(Atom::from("0"))
                                                    }))
                                                ))
                                            ))
                                        );


                                        updated_body.push(ModuleItem::from(void_0_statement));
                                    }
                                },
                                _ => {}
                            }

                            match node_declaration.clone() {
                                Decl::Class(class_declaration) => {
                                    let export_name = class_declaration.ident.sym.clone();

                                    updated_body.push(ModuleItem::from(Stmt::Decl(Decl::from(class_declaration))));
                                    updated_body.push(ModuleItem::from(define_export_assignment_by_identfier(&export_name, &export_name)))
                                },
                                Decl::Fn(function_declaration) => {
                                    let export_name = function_declaration.ident.sym.clone();

                                    updated_body.push(ModuleItem::from(Stmt::Decl(Decl::from(function_declaration))));
                                    updated_body.push(ModuleItem::from(define_export_assignment_by_identfier(&export_name, &export_name)))
                                },
                                Decl::Var(var_declaration) => {

                                    for ele in var_declaration.decls {
                                        let export_name = ele.name.as_ident().unwrap().sym.clone();


                                        match ele.init {
                                            Some(init) => {
                                                updated_body.push(ModuleItem::from(create_expression_statement_as_stmt(
                                                    Box::new(create_assignment_expression(
                                                        AssignOp::Assign,
                                                        PatOrExpr::Expr(Box::new(Expr::from(create_member_expression(
                                                            Expr::from(create_identifier("exports")),
                                                            &export_name
                                                        )))),
                                                        init
                                                    ))
                                                )))
                                            },
                                            _ => {}
                                        }
                                    };
                                },
                                _ => {}
                            };
                        }
                        if var.is_export_default_decl() {
                            if self.has_writed_name_exports_module == false {
                                if self.contains_use_strict_declaration == false {
                                    let use_strict = create_js_string_as_stmt("use strict");
                                    updated_body.push(ModuleItem::from(use_strict));
                                    self.contains_use_strict_declaration = true;
                                }

                                if self.contains_es_module_definition == false {
                                    let define_es_module_value = define_es_module_property(true);
                                    updated_body.push(ModuleItem::from(define_es_module_value));
                                    self.contains_es_module_definition = true;
                                }

                                self.has_writed_name_exports_module = true;
                            }

                            match var.as_export_default_decl() {
                                Some(node_declaration) => {
                                    match node_declaration.clone().decl {
                                        DefaultDecl::Class(class_declaration) => {
                                            let export_name = match class_declaration.ident.clone() {
                                                Some(ident) => ident.sym.clone(),
                                                _ => {
                                                    self.class_default_number = self.class_default_number + 1;
                                                    JsWord::from(format!("default_class_{}", self.class_default_number))
                                                }
                                            };

                                            updated_body.push(ModuleItem::from(Stmt::Decl(Decl::Class(ClassDecl {
                                                ident: create_identifier(&export_name),
                                                declare: false,
                                                class: class_declaration.class
                                            }))));
                                            updated_body.push(ModuleItem::from(define_export_assignment_by_identfier("default", &export_name)))
                                        },
                                        DefaultDecl::Fn(function_declaration) => {
                                            let export_name = match function_declaration.ident.clone() {
                                                Some(ident) => ident.sym.clone(),
                                                _ => {
                                                    self.function_default_number = self.function_default_number + 1;
                                                    JsWord::from(format!("default_function_{}", self.function_default_number))
                                                }
                                            };

                                            updated_body.push(ModuleItem::from(Stmt::Decl(Decl::Fn(FnDecl {
                                                ident: create_identifier(&export_name),
                                                declare: false,
                                                function: function_declaration.function
                                            }))));
                                            updated_body.push(ModuleItem::from(define_export_assignment_by_identfier("default", &export_name)))
                                        }
                                        _ => {}
                                    }
                                }
                                None => {}
                            }
                        }
                        if var.is_export_default_expr() {
                            if self.has_writed_name_exports_module == false {
                                if self.contains_use_strict_declaration == false {
                                    let use_strict = create_js_string_as_stmt("use strict");
                                    updated_body.push(ModuleItem::from(use_strict));
                                    self.contains_use_strict_declaration = true;
                                }

                                if self.contains_es_module_definition == false {
                                    let define_es_module_value = define_es_module_property(true);
                                    updated_body.push(ModuleItem::from(define_es_module_value));
                                    self.contains_es_module_definition = true;
                                }

                                self.has_writed_name_exports_module = true;
                            }

                            let node_expression = var.as_export_default_expr().unwrap();

                            updated_body.push(ModuleItem::from(define_export_assignment_by_literal_value(
                                "default",
                                node_expression.expr.clone()
                            )))
                        }
                    },
                    _ => {

                    }
                }
            }

            module.body = updated_body;

            self.has_writed_all_module = old_has_writed_all_module;
            self.has_writed_name_exports_module = old_has_writed_name_exports_module;
        }
    }
