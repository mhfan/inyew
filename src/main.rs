use yew::prelude::*;
use yew_router::prelude::*;

// ===================================================================================
// for {username}.github.io/{repo_name}
// replace 'yew-template-for-github.io' to your repo name

#[derive(Clone, Routable, PartialEq)] enum RootRoute {
    #[at("/inyew/")] Home,
    #[at("/inyew/:s")] Route,
}

#[derive(Clone, Routable, PartialEq)] enum Route {
    #[at("/inyew/about")] About,
    #[at("/inyew/404")] #[not_found] NotFound,
}

use web_sys::HtmlInputElement;
use {wasm_bindgen::JsCast, js_sys::Function};
use std::collections::VecDeque;

struct Game24 {
    goal: i32,
    nums: Vec<i32>,

    deck: Vec<i32>,
    pos: usize,
    cnt: usize,

    elem_op: Option<HtmlInputElement>,
    elem_nq: VecDeque<HtmlInputElement>,
    dblclick: Option<Function>,
}

impl Game24 {
    fn dealer(&mut self, n: usize) {
        use rand::{thread_rng, seq::SliceRandom};
        let mut rng = thread_rng();

        if self.deck.len() < self.pos + n { self.pos = 0; }
        if self.pos == 0 { self.deck.shuffle(&mut rng); }

        self.nums = self.deck[self.pos..].partial_shuffle(&mut rng, n).0.to_owned();
        self.pos += n;  // TODO: solvable assurance
    }

    fn combine(&mut self) {
        let nq = &mut self.elem_nq;
        let op = self.elem_op.as_ref().unwrap();
        let str = format!("({} {} {})", nq[0].value(), op.value(), nq[1].value());
        log::info!("{str}");

        nq[1].set_size (str.len() as u32);  nq[1].set_max_length(str.len() as i32);
        nq[1].set_value(str.as_str());      nq[0].set_hidden(true);

        self.cnt += 1;  if self.cnt == 2 {
            let parent = nq[0].parent_element().unwrap()
                    .dyn_into::<web_sys::HtmlElement>().unwrap();
            self.dblclick = parent.ondblclick();
            if self.dblclick.is_some() { parent.set_ondblclick(None); }
            else { log::warn!("dblclick is none"); }   // FIXME:
        } else if self.cnt == self.nums.len() {
            Self::toggle_hl(&nq[1], false);  nq[1].blur().unwrap();
            // TODO: calculate expression in str, reflect result in equal button
        }

        op.set_checked(false);  self.elem_op = None;
        Self::toggle_hl(&nq.pop_front().unwrap(), false);
    }

    fn clear_state(&mut self) { log::info!("clear state");
        self.elem_nq.iter().for_each(|el| Self::toggle_hl(el, false));
        self.elem_nq.clear();   self.elem_op = None;    self.cnt = 1;

        let parent = web_sys::window().unwrap().document().unwrap()
            .get_element_by_id("num-operands").unwrap();
        let coll = parent.children();
        if self.dblclick.is_some() {
            parent.dyn_into::<web_sys::HtmlElement>().unwrap()
                .set_ondblclick(self.dblclick.as_ref());
        }

        for i in 0..coll.length() {
            let inp = coll.item(i).unwrap()
                .dyn_into::<HtmlInputElement>().unwrap();
            inp.set_max_length(3);  inp.set_size(3);
            inp.set_hidden(false);
        }
    }

    fn toggle_hl(el: &HtmlInputElement, hl: bool) {
        if hl {
            el.class_list().add_2("ring-2", "ring-purple-600").unwrap();
        } else {
            el.class_list().remove_2("ring-2", "ring-purple-600").unwrap();
        }
    }
}

enum Msg {
    Operator(HtmlInputElement),
    Operands(HtmlInputElement),
    Resize(u8),
    Restore,
}

impl Component for Game24 {
    type Properties = ();
    type Message = Msg;

    fn create(_ctx: &Context<Self>) -> Self {
        let mut game = Self { goal: 24, nums: vec![],
            deck: (0..52).collect::<Vec<_>>(), pos: 0, cnt: 1,
            elem_op: None, elem_nq: VecDeque::new(), dblclick: None,
        };  game.dealer(4);     game
    }

    //#[function_component(Game24F)] fn game24() -> Html
fn view(&self, ctx: &Context<Self>) -> Html {
    let link = ctx.link();
    let ops_selected = link.callback(|e: Event| {
        let inp = e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        log::info!("{}", inp.value());  // require value='xxx' in <input>, default is 'on'
        Msg::Operator(inp)
    });

    let cnt_changed = link.callback(|e: Event| {
        let sel = e.target().unwrap()
            .dyn_into::<web_sys::HtmlSelectElement>().unwrap();
        log::info!("{}", sel.inner_text());     //sel.value()
        Msg::Resize(sel.value().parse::<u8>().unwrap())
    });

    let restore = link.callback(|_| Msg::Restore);
    let refresh = link.callback(|_| Msg::Resize(0) );
    //web_sys::window().map(|window| window.location().reload());

    // TODO: drag to exchange/replace

    let num_editable = Callback::from(|e: MouseEvent| {
        let inp = e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        //inp.set_selection_range(end, inp.value().len() as u32).unwrap();
        inp.remove_attribute("readonly").expect("");
    });

    let num_readonly = Callback::from(|e: FocusEvent| {
        let inp = e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        if  inp.check_validity() {
            inp.set_attribute("readonly", "").unwrap();
        } else {
            inp.focus().unwrap();  inp.select();
        }   log::info!("input {}", inp.value());
    });

    let num_selected = link.callback(|e: FocusEvent| {
        let inp = e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        log::info!("{}", inp.value());
        Msg::Operands(inp)
    });

    let num_class = "px-4 py-2 m-4 w-fit bg-transparent text-center text-2xl text-purple-600 font-semibold border border-purple-200 hover:text-white hover:bg-purple-600 hover:border-transparent focus:outline-none focus:ring-2 focus:ring-purple-600 focus:ring-offset-2 shadow-xl invalid:border-red-500";
    let nums = self.nums.iter().enumerate().map(|(_i, pkn)| {
        let (num, sid) = ((pkn % 13) + 1, (pkn / 13)/* % 4 */);

        let court = [ "T", "J", "Q", "K" ];
        let suits = [ "S", "C", "D", "H" ];     // "♣♦♥♠"
        let _ = format!(r"{}{}.svg", match num { 1 => "A".to_owned(),
            2..=9 => num.to_string(), 10..=13 => court[(num - 10) as usize].to_owned(),
            _ => "?".to_owned() }, suits[sid as usize]);     //num  // TODO:

        html!{
            <input type="text" value={ num.to_string() } placeholder="?" inputmode="numeric" pattern=r"-?\d+" maxlength="3" size="3" draggable="true" readonly=true class={ classes!(num_class, "rounded-full") } data-bs-toggle="tooltip" title="Click to select/unselect\nDrag over to exchange\nDouble click to input new number"/>
            // XXX: https://en.wikipedia.org/wiki/Playing_cards_in_Unicode
        }
    }).collect::<Html>();

    let ops = [ "+", "-", "×", "÷" ].into_iter().map(|op| html!{ <div class="m-4">
            <input type="radio" id={ op } value={ op } class="hidden peer"/>
            <label for={ op } draggable="true" class="px-4 py-2 m-4 bg-indigo-600 text-white text-3xl font-bold hover:bg-indigo-500 peer-checked:outline-none peer-checked:ring-2 peer-checked:ring-indigo-500 peer-checked:ring-offset-2 peer-checked:bg-transparent rounded-md shadow-xl" data-bs-toggle="tooltip" title="Click to select/unselect\nDrag over to replace">{ op }</label>
        </div>
    }).collect::<Html>();

    let ctrl_class = "px-4 py-2 m-4 text-gray-900 font-bold bg-gradient-to-r from-lime-200 via-lime-400 to-lime-500 rounded-lg hover:bg-gradient-to-br focus:ring-4 focus:outline-none focus:ring-lime-300 dark:focus:ring-lime-800 shadow-lg shadow-lime-500/50 dark:shadow-lg dark:shadow-lime-800/80";

    let cnt_options = (4..=6).map(|n| html!{
        <option value={ n.to_string() } selected={ n == self.nums.len() }>{ format!("{n} nums") }</option>
    }).collect::<Html>();

    // https://stackoverflow.com/questions/62554142/how-to-select-multiple-button-options-on-same-html-page-using-tailwind-css
    html!{ <main>
        //<div id="play-cards"/>    // TODO:

        <div id="ops-group" onchange={ ops_selected } class="flex place-content-center">{ ops }</div>

        <div id="expr-skel" class="flex place-content-center">
            <style>{ r"
                [contenteditable='true'].single-line { white-space: nowrap; overflow: hidden; }
                [contenteditable='true'].single-line br { display: none; }
                [contenteditable='true'].single-line  * { display: inline; white-space: nowrap; }
            " }</style>

            <div id="num-operands" class="flex place-content-center" onfocus={ num_selected } ondblclick={ num_editable.clone() } onblur={ num_readonly.clone() }>{ nums }</div>

            // data-bs-toggle="collapse" data-bs-target="#all-solutions" aria-expanded="false" aria-controls="all-solutions"
            <button class="py-2 px-4 m-4 text-white text-3xl font-bold rounded-md hover:outline-none hover:ring-2 focus:ring-indigo-500 active:ring-offset-2" data-bs-toggle="tooltip" title="Click to show solutions">{ "≠?" }</button>
            <input type="text" value={ self.goal.to_string() } placeholder="??" inputmode="numeric" pattern=r"-?\d+" maxlength="4" size="4" readonly=true ondblclick={ num_editable } onblur={ num_readonly } class={ classes!(num_class, "rounded-md") } data-bs-toggle="tooltip" title="Double click to input new goal"/>
        </div>

        <div id="ctrl-btns">
            <select class={ classes!(ctrl_class) } onchange={ cnt_changed } data-bs-toogle="tooltip" title="Click to select numbers count">{ cnt_options }</select>
            <input type="reset" value={ "Restore" } onclick={ restore } class={ classes!(ctrl_class) } data-bs-toogle="tooltip" title="Click to break down selected compound expression"/> //{ "Restore" } </button>
            <button class={ classes!(ctrl_class) } onclick={ refresh } data-bs-toogle="tooltip" title="Click to refresh new round game">{ "Refresh" }</button>
        </div>

        <div id="all-solutions" class="collapse block mt-4 max-h-24 overflow-y-auto text-white text-2xl">{ "All solutions" }
        </div>
    </main> }
}

    fn update  (&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Operator(inp) => { self.elem_op = Some(inp);
                if  self.elem_nq.len() == 2 { self.combine(); }     false
            }

            Msg::Operands(inp) => {
                let nq = &mut self.elem_nq;
                let mut n = nq.len();
                if  nq.iter().enumerate().any(|(i, el)| {
                    let same = el.is_same_node(Some(inp.as_ref()));
                    if same { n = i; }  same }) {
                    Self::toggle_hl(&inp, false);
                    if n < nq.len() { nq.remove(n); }
                } else {
                    Self::toggle_hl(&inp, true);
                    nq.push_back(inp);
                }

                if 2 < nq.len() {
                    Self::toggle_hl(&nq.pop_front().unwrap(), false);
                }

                if  nq.len() == 2 && self.elem_op != None { self.combine(); }   false
            }

            Msg::Resize(n) => {    self.clear_state();
                if 0 < n { self.dealer(n as usize); } else {
                           self.dealer(self.nums.len());
                }   true
            }

            Msg::Restore => { self.clear_state();   true }
        }
    }

    //fn changed (&mut self, ctx: &Context<Self>) -> bool { true }

    //fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {}

    //fn destroy (&mut self, ctx: &Context<Self>) {}
}

fn root_route(routes: &RootRoute) -> Html {
    #[allow(clippy::let_unit_value)]
    match routes {
        RootRoute::Home  => html!{ <>
            //margin: 0 auto;   //class: justify-center;    // XXX: not working
            <style>{ r" body { text-align: center; } " }</style>

            <header>
            <br/> <h1 class="text-4xl"><a href="https://github.com/mhfan/inrust">{ "24 Game/Puzzle/Challenge" }</a></h1> <br/>
            </header>

            <Game24 />

            //<footer> <br/><p>{ "Copyright © 2022, " }<a href="https://github.com/mhfan">{ "mhfan" }</a></p><br/> </footer>   // &copy;
        </> },

        RootRoute::Route => html!{ <Switch<Route> render={Switch::render(switch)} /> },
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::About => html!{ <p>{ "About" }</p> },
        Route::NotFound => html!{ <p>{ "Not Found" }</p> },
    }
}

/* ===================================================================================
// for {username}.github.io

#[derive(Clone, Routable, PartialEq)] enum RootRoute {
    #[at("/")] Home,
    #[at("/about")] About,
    #[at("/404")] #[not_found] NotFound,
}

fn root_route(routes: &Route) -> Html {
    match routes {
        RootRoute::Home => html!{ <p class="text-4xl">{ "Yew Template" }</p> },
        RootRoute::About => html!{ <p>{ "About" }</p> },
        RootRoute::NotFound => html!{ <p>{ "Not Found" }</p> },
    }
}

// =================================================================================== */

#[function_component(App)] fn app() -> Html {   // main root
    html!{
        /********************************************************
         **    basename is not supported on yew 0.19.0 yet.    **
        <BrowserRouter basename="/inyew/">
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
        // ******************************************************/
        <BrowserRouter>
            <Switch<RootRoute> render={Switch::render(root_route)} />
        </BrowserRouter>
    }
}

fn main() {     // entry point
    wasm_logger::init(wasm_logger::Config::default()); //log::info!("Update: {:?}", msg);
    yew::start_app::<App>();
}
