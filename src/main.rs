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

use web_sys::{HtmlInputElement, HtmlSelectElement}; // HtmlButtonElement
use rand::{rngs::ThreadRng, thread_rng, seq::SliceRandom};

struct Game24 {
    goal: i32,
    nums: Vec<i32>,
    poker: Vec<i32>,
    pos: usize,
    rng: ThreadRng,

    elem_op: Option<HtmlInputElement>,
    elem_na: Option<HtmlInputElement>,
}

enum Msg {
    Operator(HtmlInputElement),
    Operands(HtmlInputElement),
    Resize(u8)
}

impl Component for Game24 {
    type Properties = ();
    type Message = Msg;

    fn create(_ctx: &Context<Self>) -> Self {
        let mut poker = (0..52).collect::<Vec<_>>();
        let mut pos = 0;

        let mut rng = thread_rng();     poker.shuffle(&mut rng);
        let nums = poker[pos..].partial_shuffle(&mut rng, 4).0.to_owned();
        pos += nums.len();

        Self { goal: 24, nums, poker, pos, rng, elem_na: None, elem_op: None, }
    }

    //#[function_component(Game24F)] fn game24() -> Html
fn view(&self, ctx: &Context<Self>) -> Html {
    let link = ctx.link();
    let ops_selected = link.callback(|e: Event| {
        // You must KNOW target is a HtmlInputElement, otherwise
        // the call to value would be Undefined Behaviour (UB).
        //e.target_unchecked_into::<HtmlInputElement>().value();
        //e.target().unwrap().unchecked_into::<HtmlInputElement>().value();

        // When events are created the target is undefined, it's only
        // when dispatched does the target get added.
        // Events can bubble so this listener might catch events from child
        // elements which are not of type HtmlInputElement
        //e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        //    .map(|inp| inp.value());

        let inp = e.target_dyn_into::<HtmlInputElement>().unwrap();
        //inp.value()   // add value='xxx' in <input> element to change default('on')
        log::info!("{}", inp.get_attribute("id").unwrap());
        Msg::Operator(inp)
    });

    let _ops_chkclear = Callback::from(|e: MouseEvent| {
        let inp = e.target_dyn_into::<HtmlInputElement>().unwrap();
        log::info!("checked: {}", inp.checked());
        if inp.checked() { inp.set_checked(false); }
    });

    let cnt_changed = link.callback(|e: Event| {
        let sel = e.target_dyn_into::<HtmlSelectElement>().unwrap();
        log::info!("{}", sel.value());  // sel.inner_text()
        Msg::Resize(sel.value().parse::<u8>().unwrap())
    });

    let new_refresh = Callback::from(|_| {
        web_sys::window().map(|window| window.location().reload());
    });

    // TODO: drag to exchange/replace

    let num_writable = Callback::from(|e: MouseEvent| {
        //if let Ok(Some(sel)) = web_sys::window().unwrap().document().unwrap()
        //    .get_selection() { sel.remove_all_ranges().expect(""); }
        let inp = e.target_dyn_into::<HtmlInputElement>().unwrap();
        let end = inp.value().len() as u32;
        inp.set_selection_range(end, end).expect("");   //inp.focus().expect("");
        inp.remove_attribute("readonly").expect("");
    });

    let num_readonly = Callback::from(|e: FocusEvent| {
        let inp = e.target_dyn_into::<HtmlInputElement>().unwrap();
        inp.set_attribute("readonly", "").expect("");
    });

    let num_selected = link.callback(|e: FocusEvent| {
        let inp = e.target_dyn_into::<HtmlInputElement>().unwrap();
        log::info!("{}", inp.value());
        Msg::Operands(inp)
    });

    let num_class = "px-4 py-2 m-4 min-w-16 w-fit bg-transparent text-center text-2xl text-purple-600 font-semibold border border-purple-200 hover:text-white hover:bg-purple-600 hover:border-transparent focus:outline-none focus:ring-2 focus:ring-purple-600 focus:ring-offset-2 shadow-xl";
    let nums = self.nums.iter().map(|pkn| {
        let (num, sid) = ((pkn % 13) + 1, (pkn / 13)/* % 4 */);

        let court = [ "T", "J", "Q", "K" ];
        let suits = [ "S", "C", "D", "H" ];     // "♣♦♥♠"
        let _ = format!(r"{}{}.svg", match num { 1 => "A".to_owned(),
            2..=9 => num.to_string(), 10..=13 => court[(num - 10) as usize].to_owned(),
            _ => "?".to_owned() }, suits[sid as usize]);     //num

        html!{  // XXX:
            <input type="text" value={ num.to_string() } placeholder="?" inputmode="numeric" pattern=r"-?\d+" maxlength="3" size="3" draggable="true" readonly=true class={ classes!(num_class, "rounded-full") } data-bs-toggle="tooltip" title="Click to select/unselect\nDrag over to exchange\nDouble click to input new number"/>
            // XXX: https://en.wikipedia.org/wiki/Playing_cards_in_Unicode
        }
    }).collect::<Html>();

    let ops = [ "+", "-", "×", "÷" ].into_iter().map(|op| html!{
        <div class="m-4">
            <input type="radio" id={ op } name="select-ops" class="hidden peer"/>
            <label for={ op } draggable="true" class="bg-indigo-600 hover:bg-indigo-500 text-white font-bold py-2 px-4 text-3xl peer-checked:outline-none peer-checked:ring-2 peer-checked:ring-indigo-500 peer-checked:ring-offset-2 peer-checked:bg-transparent rounded-md shadow-xl" data-bs-toggle="tooltip" title="Click to select/unselect\nDrag over to replace">{ op }</label>
        </div>
    }).collect::<Html>();

    let ctrl_class = "text-gray-900 bg-gradient-to-r from-lime-200 via-lime-400 to-lime-500 hover:bg-gradient-to-br focus:ring-4 focus:outline-none focus:ring-lime-300 dark:focus:ring-lime-800 shadow-lg shadow-lime-500/50 dark:shadow-lg dark:shadow-lime-800/80 font-bold rounded-lg px-4 py-2 m-4";

    let cnt_options = (4..=6).map(|n| html!{
        <option value={ n.to_string() } selected={ n == self.nums.len() }>{ format!("{n} nums") }</option>
    }).collect::<Html>();

    // https://stackoverflow.com/questions/62554142/how-to-select-multiple-button-options-on-same-html-page-using-tailwind-css
    html!{ <main>
        //<div id="play-cards"/>    // TODO:

        <div id="ops-group" onchange={ ops_selected } class="flex place-content-center">{ ops }</div>

        <div id="expr-skel select-none" user-select="none">     // XXX: why not working?
            <span onfocus={ num_selected } ondblclick={ num_writable.clone() } onblur={ num_readonly.clone() }>{ nums }</span>
            <label class="text-white font-bold py-2 px-4 m-4 text-3xl rounded-md" data-bs-toggle="tooltip" title="Click to calculate">{ "≠?" }</label>
            <input type="text" value={ self.goal.to_string() } placeholder="??" inputmode="numeric" pattern=r"-?\d+" maxlength="4" size="4" readonly=true name="goal" ondblclick={ num_writable } onblur={ num_readonly } class={ classes!(num_class, "rounded-md") } data-bs-toggle="tooltip" title="Double click to input new goal"/>
        </div>

        <div id="ctrl-btns">
            <select class={ classes!(ctrl_class) } onchange={ cnt_changed } data-bs-toogle="tooltip" title="Click to select numbers count">{ cnt_options }</select>
            <input type="reset" value={ "Restore" } class={ classes!(ctrl_class) } data-bs-toogle="tooltip" title="Click to break down selected compound expression"/> //{ "Restore" } </button>    // XXX:
            <button class={ classes!(ctrl_class) } onclick={ new_refresh } data-bs-toogle="tooltip" title="Click to refresh new round game">{ "Refresh" }</button>
        </div>
    </main> }
}

    fn update  (&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Operator(inp) => { self.elem_op = Some(inp);   false }

            Msg::Operands(inp) => {
                if self.elem_na != None && self.elem_op != None {
                    let na = self.elem_na.as_ref().unwrap();
                    let op = self.elem_op.as_ref().unwrap();
                    if  na.is_same_node(Some(inp.as_ref())) { return false }

                    let str = format!("{} {} {}", na.value(),
                        op.get_attribute("id").unwrap(), inp.value());
                    inp.set_value(str.as_str());    log::info!("{str}");    // FIXME:
                    //inp.parent_element().unwrap().remove_child(elem_na.unwrap().as_ref());
                    op.set_checked(false);  na.set_hidden(true);

                    (self.elem_op, self.elem_na) = (None, Some(inp));
                } else { self.elem_na = Some(inp); }    false
            }

            Msg::Resize(cnt) => {   let cnt = cnt as usize;
                if self.poker.len()  < self.pos + cnt { self.pos = 0; }
                self.nums = self.poker[self.pos..].partial_shuffle(&mut self.rng,
                    cnt).0.to_owned();
                self.pos += self.nums.len();    true
            }
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
            <br/> <h1 class="text-4xl">{ "24 Game/Puzzle/Challenge" }</h1> <br/>
            </header>

            <Game24 />

            //<footer> <br/><p>{ "Copyright © 2022, mhfan" }</p><br/> </footer>   // &copy;
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
