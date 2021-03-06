use crate::common::MaidenError;
use crate::display;
use crate::parser;
use crate::runner;
use std;
use stdweb::js;
use yew::html;
use yew::prelude::*;
use yew::services::ConsoleService;

pub struct Model {
    value: String,
    program: String,
    parse_error: bool,
    res: String,
    run_error: bool,
    input_callback: Callback<String>,
    console: ConsoleService,
}

pub enum Msg {
    GotInput(String),
}

impl Model {
    fn get_line(&self, line: usize) -> &str {
        if line == 0 {
            &self.value
        } else {
            self.value.split("\n").nth(line - 1).unwrap()
        }
    }

    fn nicer_error(&self, err: &MaidenError) -> String {
        let line = get_error_line(err);
        if line == 0 {
            format!("{}", err)
        } else {
            format!(
                "{} at line {}: \"{}\"",
                err,
                line,
                self.get_line(line as usize)
            )
        }
    }

    fn ast_tab(&self) -> Html<Self> {
        if self.parse_error {
            html! {
                { &self.program }
            }
        } else {
            html! {
                <pre> { &self.program } </pre>
            }
        }
    }

    fn run_program(&mut self) {
        let program = parser::parse(&self.value);
        match program {
            Err(err) => {
                self.program = self.nicer_error(&err);
                self.parse_error = true;
                self.res = "".to_string()
            }
            Ok(mut val) => {
                self.program = display::print_program(&val);
                self.parse_error = false;
                let mut writer = std::io::Cursor::new(Vec::new());
                let res = runner::run(&mut val, &mut writer);
                self.res = "".into();
                if let Err(err) = res {
                    self.res += &self.nicer_error(&err);
                    self.run_error = true;
                } else {
                    self.run_error = false;
                }
                writer.set_position(0);
                self.res += std::str::from_utf8(writer.get_ref()).unwrap().into();
                if self.res.is_empty() {
                    self.res = "<No output from program>".to_string();
                }
            }
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut res = Self {
            value: include_str!("../tests/local/modulo.rock").into(),
            program: "".into(),
            parse_error: false,
            res: "".into(),
            run_error: false,
            input_callback: link.send_back(|data| Msg::GotInput(data)),
            console: ConsoleService::new(),
        };
        res.run_program();
        res
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GotInput(input_data) => {
                self.console.log("Change");
                self.value = input_data;
                self.run_program();
                true
            }
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        let callback = self.input_callback.clone();
        let do_input = move |payload: String| callback.emit(payload);
        js! {
            function codeMirrorCallback() {
                if (window.codeMirror) {
                    if (!window.codeMirror.configured) {
                        window.codeMirror.on("change", function(cm, change) {
                            var callback = @{do_input};
                            callback(cm.getValue());
                        });
                        window.codeMirror.setValue(@{&self.value});
                        console.log("setup callback");
                        window.codeMirror.configured = true;
                    }
                }
                else {
                    window.setTimeout(codeMirrorCallback, 500);
                }
            }
            codeMirrorCallback();
        }
        html! {
            <div class="row",>
                <div class="col-xl-6",>
                    <textarea id="editor",
                        class="form-control",
                        value=&self.value,
                        placeholder="placeholder",>
                    </textarea>
                </div>
                <div class="col-xl-6",>
                    <ul class=("nav", "nav-tabs"), id="outputTabs", role="tablist",>
                        <li class="nav-item",>
                            <a class=("nav-link", "active"),
                                id="ast-tab", data-toggle="tab",
                                href="#ast", role="tab",
                                style=if self.parse_error {"color: red"} else {"color: green"},
                                aria-controls="ast", aria-selected="true",>{ "AST" }</a>
                        </li>
                        <li class="nav-item",
                            id="output-tab-li",
                            style=if self.parse_error {"display: none"} else {""},>
                            <a class="nav-link", id="output-tab",
                                data-toggle="tab", href="#output",
                                role="tab", aria-controls="output",
                                style=if self.run_error {"color: red"} else {"color: green"},
                                aria-selected="false",>{ "Output" }</a>
                        </li>
                    </ul>
                    <div class="tab-content", id="outputTabsContent",>
                        <div class=("tab-pane", "fade", "show", "active"),
                            id="ast", role="tabpanel", aria-labelledby="ast-tab",>
                            {self.ast_tab()}
                        </div>
                        <div class=("tab-pane", "fade"),
                            id="output",
                            style=if self.parse_error {"display: none"} else {""},
                            role="tabpanel", aria-labelledby="output-tab",>
                            <pre>{&self.res}</pre>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

fn get_error_line(e: &MaidenError) -> usize {
    match e {
        MaidenError::MissingVariable { ref line, .. }
        | MaidenError::MissingFunction { ref line, .. }
        | MaidenError::WrongArgCount { ref line, .. }
        | MaidenError::ParseNumberError { ref line, .. }
        | MaidenError::NoEndOfIf { ref line }
        | MaidenError::BadBooleanResolve { ref line, .. }
        | MaidenError::Unimplemented { ref line, .. }
        | MaidenError::StackOverflow { ref line, .. }
        | MaidenError::InstructionLimit { ref line }
        | MaidenError::UndefinedPronoun { ref line }
        | MaidenError::Infinity { ref line, .. }
        | MaidenError::Incomplete { ref line, .. }
        | MaidenError::NotAnExpression { ref line, .. }
        | MaidenError::NotASymbol { ref line, .. }
        | MaidenError::NotACommand { ref line, .. }
        | MaidenError::NotABlock { ref line, .. }
        | MaidenError::BadString { ref line, .. } => *line,
        MaidenError::Pest { .. } | MaidenError::Io { .. } => 0,
    }
}
