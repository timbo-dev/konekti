use ast_transformer::AstTransformer;
use export_decl::ExportDeclVisitor;
use export_default_decl::ExportDefaultDeclVisitor;
use export_default_expr::ExportDefaultExprVisitor;
use export_default_ident::ExportDefaultIdentVisitor;

use swc_core::plugin::{
    plugin_transform,
    proxies::TransformPluginProgramMetadata
};

use swc_core::ecma::{
    ast::*,
    visit::VisitMut,
    atoms::JsWord
};

use swc_core::common::DUMMY_SP;
use swc_core::ecma::visit::VisitMutWith;
use swc_ecma_utils::quote_ident;

use swc_ecma_quote::{
    quote,
    quote_expr
};

mod ast_transformer;

mod export_decl;
mod export_default_decl;
mod export_default_expr;
mod export_default_ident;

pub struct ModuleExportsVisitor {
    updated_body: Vec<ModuleItem>,

    export_default_decl_index: ExportDefaultDeclVisitor,
    export_default_expr_index: ExportDefaultExprVisitor,
    export_default_ident_index: ExportDefaultIdentVisitor,

    has_void_stmt: bool,
    has_module_exports_stmt: bool
}

impl ModuleExportsVisitor {
    fn append_module_exports_stmt(&mut self) {
        if !self.has_module_exports_stmt {
            self.has_module_exports_stmt = true;
            self.updated_body.insert(PROGRAM_TOP_INDEX, quote!(
                r#"Object.defineProperty(exports, "__esModule", { value: true });"#
            as ModuleItem));
        }
    }

    fn append_export_stmt_with_value_as_ident(&mut self, ident: &mut Ident) {
        self.updated_body.insert(
            self.export_decl_visitor.get(),
            quote!(
                "exports.$ident = $value_as_ident;" as ModuleItem,
                ident = ident.clone(),
                value_as_ident = ident.clone()
            )
        )
    }

    fn append_export_stmt_with_value(&mut self, ident: &mut Ident, value: &Box<Expr>, position: usize) {
        self.updated_body.insert(
            position,
            quote!(
                "exports.$ident = $value;" as ModuleItem,
                ident = ident.clone(),
                value: Expr = *value.clone(),
            )
        )
    }

    fn append_void_stmt(&mut self, ident: &Ident) {
        fn create_void_stmt(ident: &Ident) -> Box<Expr> {
            quote_expr!(
                "exports.$name = void 0;",
                name = ident.clone()
            )
        }

        if !self.has_void_stmt {
            self.has_void_stmt = true;

            self.updated_body.insert(PROGRAM_VOID_STMT_INDEX, ModuleItem::Stmt(Stmt::Expr(ExprStmt {
                span: DUMMY_SP,
                expr: create_void_stmt(ident)
            })));
        } else {
            if let ModuleItem::Stmt(Stmt::Expr(ref mut expr_stmt)) = &mut self.updated_body[PROGRAM_VOID_STMT_INDEX] {
                if let Expr::Assign(ref mut assign_expr) = &mut *expr_stmt.expr {
                    let mut previous_assign_expr = assign_expr;

                    loop {
                        if let Expr::Assign(ref mut next_assign_expr) = *previous_assign_expr.right {
                            previous_assign_expr = next_assign_expr;
                        } else {
                            previous_assign_expr.right = create_void_stmt(ident);
                            break;
                        }
                    }
                }
            }
        }
    }

    fn update_module_body(&self, module_to_update: &mut Module) {
        module_to_update.body = self.updated_body.clone();
    }

    fn get_module_item_position(&self, module_item: &ModuleItem) -> usize {
        let search_result       = self.updated_body
            .iter()
            .position(|node| module_item == node);

        search_result.unwrap()
    }
}

pub fn create_module_exports_visitor() -> impl VisitMut {
    ModuleExportsVisitor {
        updated_body: vec![],

        export_default_decl_index: ExportDefaultDeclVisitor { index: 0 },
        export_default_expr_index: ExportDefaultExprVisitor { index: 0 },
        export_default_ident_index: ExportDefaultIdentVisitor { index: 0 },

        has_void_stmt: false,
        has_module_exports_stmt: false,
    }
}

impl VisitMut for ModuleExportsVisitor {
    fn visit_mut_module(&mut self, module: &mut Module) {
        let initial_body = module.body.clone();
        self.updated_body = initial_body;

        module.visit_mut_children_with(self);

        self.update_module_body(module);
    }

    fn visit_mut_export_decl(&mut self, export_decl: &mut ExportDecl) {
        let export_decl_visitor = ExportDeclVisitor {
            ast_transformer: AstTransformer {
                index: 0,
                updated_body: &mut self.updated_body,
                has_void_stmt: &mut self.has_void_stmt,
                has_module_exports_stmt: &mut self.has_module_exports_stmt
            },
            node: export_decl
        };

        let initial_export_decl_position = self.get_module_item_position(&ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl.clone())));
        self.export_decl_visitor.init(initial_export_decl_position);

        if !self.has_module_exports_stmt {
            self.export_decl_visitor.increment();
        }

        self.append_module_exports_stmt();

        match &mut export_decl.decl {
            Decl::Class(class_decl) => {
                let ident = &mut class_decl.ident.clone();

                if !self.has_void_stmt {self.export_decl_visitor.increment();}
                self.append_void_stmt(ident);

                self.updated_body.insert(
                    self.export_decl_visitor.get(),
                    ModuleItem::Stmt(Stmt::Decl(Decl::Class(
                        class_decl.clone()
                    )))
                );

                self.export_decl_visitor.increment();

                self.append_export_stmt_with_value_as_ident(ident);
                self.export_decl_visitor.increment()
            }

            Decl::Fn(func_decl) => {
                let ident = &mut func_decl.ident.clone();

                if !self.has_void_stmt { self.export_decl_visitor.increment() }
                self.append_void_stmt(ident);

                self.updated_body.insert(
                    self.export_decl_visitor.get(),
                    ModuleItem::Stmt(Stmt::Decl(Decl::Fn(
                        func_decl.clone()
                    )))
                );

                self.export_decl_visitor.increment();

                self.append_export_stmt_with_value_as_ident(ident);
                self.export_decl_visitor.increment()
            }

            Decl::Var(var_decl) => {
                for decl in &mut var_decl.decls {
                    if let Pat::Ident(ref decl_binding_ident) = decl.name {
                        let ident = &mut decl_binding_ident.id.clone();

                        if !self.has_void_stmt { self.export_decl_visitor.increment() }
                        self.append_void_stmt(ident);

                        if let Option::Some(some_init) = &decl.init {

                            fn create_var_decl(parent_self: &mut ModuleExportsVisitor, element: ModuleItem) {
                                parent_self.updated_body.insert(
                                    parent_self.export_decl_visitor.get(),
                                    element
                                )
                            }

                            match &**some_init {
                                Expr::Class(class_expr) => {
                                    match var_decl.kind {
                                        VarDeclKind::Const => create_var_decl(
                                            self,
                                             quote!(
                                                "const $ident = $value;" as ModuleItem,
                                                ident = ident.clone(),
                                                value: Expr = Expr::Class(class_expr.clone())
                                            )),
                                        VarDeclKind::Let => create_var_decl(
                                            self,
                                             quote!(
                                                "let $ident = $value;" as ModuleItem,
                                                ident = ident.clone(),
                                                value: Expr = Expr::Class(class_expr.clone())
                                            )),
                                        VarDeclKind::Var => create_var_decl(
                                            self,
                                             quote!(
                                                "var $ident = $value;" as ModuleItem,
                                                ident = ident.clone(),
                                                value: Expr = Expr::Class(class_expr.clone())
                                            )),
                                    }

                                    self.export_decl_visitor.increment();

                                    self.append_export_stmt_with_value_as_ident(ident);
                                    self.export_decl_visitor.increment()
                                }

                                Expr::Fn(func_expr) => {
                                    match var_decl.kind {
                                        VarDeclKind::Const => create_var_decl(
                                            self,
                                             quote!(
                                                "const $ident = $value;" as ModuleItem,
                                                ident = ident.clone(),
                                                value: Expr = Expr::Fn(func_expr.clone())
                                            )),
                                        VarDeclKind::Let => create_var_decl(
                                            self,
                                             quote!(
                                                "let $ident = $value;" as ModuleItem,
                                                ident = ident.clone(),
                                                value: Expr = Expr::Fn(func_expr.clone())
                                            )),
                                        VarDeclKind::Var => create_var_decl(
                                            self,
                                             quote!(
                                                "var $ident = $value;" as ModuleItem,
                                                ident = ident.clone(),
                                                value: Expr = Expr::Fn(func_expr.clone())
                                            )),
                                    }

                                    self.export_decl_visitor.increment();

                                    self.append_export_stmt_with_value_as_ident(ident);
                                    self.export_decl_visitor.increment()
                                }

                                Expr::Arrow(arrow_expr) => {
                                    match var_decl.kind {
                                        VarDeclKind::Const => create_var_decl(
                                            self,
                                             quote!(
                                                "const $ident = $value;" as ModuleItem,
                                                ident = ident.clone(),
                                                value: Expr = Expr::Arrow(arrow_expr.clone())
                                            )),
                                        VarDeclKind::Let => create_var_decl(
                                            self,
                                             quote!(
                                                "let $ident = $value;" as ModuleItem,
                                                ident = ident.clone(),
                                                value: Expr = Expr::Arrow(arrow_expr.clone())
                                            )),
                                        VarDeclKind::Var => create_var_decl(
                                            self,
                                             quote!(
                                                "var $ident = $value;" as ModuleItem,
                                                ident = ident.clone(),
                                                value: Expr = Expr::Arrow(arrow_expr.clone())
                                            )),
                                    }

                                    self.export_decl_visitor.increment();

                                    self.append_export_stmt_with_value_as_ident(ident);
                                    self.export_decl_visitor.increment()
                                }

                                _ => {
                                    self.append_export_stmt_with_value(
                                        ident,
                                        some_init,
                                        self.export_decl_visitor.get()
                                    );
                                    self.export_decl_visitor.increment();
                                }
                            }
                        } else {
                            // This code below is to ensure enum declarations will be correct
                            // The Decl::TsEnum match not working as expected.

                            if self.updated_body.len() > self.export_decl_visitor.get() + 1 {
                                let enum_decl = &mut self.updated_body[self.export_decl_visitor.get() + 1];

                                if let ModuleItem::Stmt(Stmt::Expr(expr_stmt)) = enum_decl {
                                    if let Expr::Call(ref mut call_expr) = *expr_stmt.expr {
                                        for arg in &mut call_expr.args {
                                            if let Expr::Bin(ref mut bin_expr) = *arg.expr {
                                                bin_expr.right = quote_expr!(
                                                    "(exports.$ident = $ident_as_value = {})",
                                                    ident = ident.clone(),
                                                    ident_as_value = ident.clone()
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            _ => {}
        }

        self.updated_body.remove(self.export_decl_visitor.get());
    }

    fn visit_mut_export_default_decl(&mut self, export_default_decl: &mut ExportDefaultDecl) {
        let initial_export_default_decl_position = self.get_module_item_position(&ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export_default_decl.clone())));
        self.export_default_decl_index.init(initial_export_default_decl_position);

        if !self.has_module_exports_stmt {
            self.export_default_decl_index.increment();
        }

        self.append_module_exports_stmt();

        match &mut export_default_decl.decl {
            DefaultDecl::Class(class_decl) => {
                if let Option::Some(ref some_ident) = class_decl.ident {
                    self.updated_body.insert(
                        self.export_default_decl_index.get(),
                        ModuleItem::Stmt(Stmt::Decl(Decl::Class(
                            ClassDecl {
                                ident: some_ident.clone(),
                                declare: false,
                                class: class_decl.class.clone()
                            }
                        )))
                    );

                    self.export_default_decl_index.increment();

                    self.append_export_stmt_with_value(
                        &mut quote_ident!("default"),
                        &Box::new(Expr::Ident(some_ident.clone())),
                        self.export_default_decl_index.get()
                    );

                    self.export_default_decl_index.increment();
                } else {
                    self.export_default_ident_index.increment();

                    let default_ident = Ident {
                        span: DUMMY_SP,
                        sym: JsWord::from(format!("default_{}", self.export_default_ident_index.get())),
                        optional: false
                    };

                    self.updated_body.insert(
                        self.export_default_decl_index.get(),
                        ModuleItem::Stmt(Stmt::Decl(Decl::Class(
                            ClassDecl {
                                ident: default_ident.clone(),
                                declare: false,
                                class: class_decl.class.clone()
                            }
                        )))
                    );

                    self.export_default_decl_index.increment();

                    self.append_export_stmt_with_value(
                        &mut quote_ident!("default"),
                        &Box::new(Expr::Ident(default_ident.clone())),
                        self.export_default_decl_index.get()
                    );

                    self.export_default_decl_index.increment();
                }
            }

            DefaultDecl::Fn(func_decl) => {
                if let Option::Some(ref some_ident) = func_decl.ident {
                    self.updated_body.insert(
                        self.export_default_decl_index.get(),
                        ModuleItem::Stmt(Stmt::Decl(Decl::Fn(
                            FnDecl {
                                ident: some_ident.clone(),
                                declare: false,
                                function:func_decl.function.clone()
                            }
                        )))
                    );

                    self.export_default_decl_index.increment();

                    self.append_export_stmt_with_value(
                        &mut quote_ident!("default"),
                        &Box::new(Expr::Ident(some_ident.clone())),
                        self.export_default_decl_index.get()
                    );

                    self.export_default_decl_index.increment();
                } else {
                    self.export_default_ident_index.increment();

                    let default_ident = Ident {
                        span: DUMMY_SP,
                        sym: JsWord::from(format!("default_{}", self.export_default_ident_index.get())),
                        optional: false
                    };

                    self.updated_body.insert(
                        self.export_default_decl_index.get(),
                        ModuleItem::Stmt(Stmt::Decl(Decl::Fn(
                            FnDecl {
                                ident: default_ident.clone(),
                                declare: false,
                                function: func_decl.function.clone()
                            }
                        )))
                    );

                    self.export_default_decl_index.increment();

                    self.append_export_stmt_with_value(
                        &mut quote_ident!("default"),
                        &Box::new(Expr::Ident(default_ident.clone())),
                        self.export_default_decl_index.get()
                    );

                    self.export_default_decl_index.increment();
                }
            }

            _ => {}
        }

        self.updated_body.remove(self.export_default_decl_index.get());
    }

    fn visit_mut_export_default_expr(&mut self, export_default_expr: &mut ExportDefaultExpr) {
        let initial_export_default_expr_position = self.get_module_item_position(&ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(export_default_expr.clone())));
        self.export_default_expr_index.init(initial_export_default_expr_position);

        if !self.has_module_exports_stmt {
            self.export_default_expr_index.increment();
        }

        self.append_module_exports_stmt();
    }
}

#[plugin_transform]
pub fn module_exports_transform(mut program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.visit_mut_with(&mut create_module_exports_visitor());
    program
}
