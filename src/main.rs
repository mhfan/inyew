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

use std::collections::VecDeque;
use web_sys::{HtmlElement, HtmlInputElement};
use wasm_bindgen::JsCast;
use inrust::calc24::*;

struct Game24 {
    goal: i32,
    nums: Vec<i32>,

    deck: Vec<i32>,
    pos: usize,
    cnt: usize,

    sol_div: NodeRef,
    num_div: NodeRef,
    elem_eq: NodeRef,

    elem_op:   Option<HtmlInputElement>,
    elem_nq: VecDeque<HtmlInputElement>,
}

impl Game24 {
    fn dealer(&mut self, n: usize) {
        use rand::{thread_rng, seq::SliceRandom};
        let mut rng = thread_rng();

        loop {
            if self.pos == 0 { self.deck.shuffle(&mut rng); }
            self.nums = self.deck[self.pos..].partial_shuffle(&mut rng,
                n).0.iter().map(|n| (n % 13) + 1).collect::<Vec<_>>();
            self.pos += n;  if self.deck.len() < self.pos + n { self.pos = 0; }

            if !calc24_first(&self.goal.into(), &self.nums.iter().map(|&n|
                Rational::from(n)).collect::<Vec<_>>(), DynProg).is_empty() { break }
        }
    }

    fn form_expr(&mut self) {
        let nq = &mut self.elem_nq;
        let op = self.elem_op.as_ref().unwrap();
        let str = format!("({} {} {})", nq[0].value(), op.value(), nq[1].value());

        nq[1].set_size (str.len() as u32);  nq[1].set_max_length(str.len() as i32);
        nq[1].set_value(&str);  nq[1].blur().unwrap();  nq[0].set_hidden(true);

        self.cnt += 1;  if self.cnt == self.nums.len() {
            let str = str.chars().map(|ch|
                match ch { '×' => '*', '÷' => '/', _ => ch }).collect::<String>();
            let elem_eq = self.elem_eq.cast::<HtmlElement>().unwrap();
            if (mexe::eval(str).unwrap() + 0.1) as i32 == self.goal {
                elem_eq.class_list().add_3("ring-2", "text-lime-500",
                    "ring-lime-400").unwrap();
                elem_eq.set_inner_text("=");
            } else {    // XXX:
                elem_eq.class_list().add_3("ring-2", "text-red-500",
                    "ring-red-400").unwrap();
                elem_eq.set_inner_text("≠");
            }
        }

        self.elem_nq.iter().for_each(|el| Self::toggle_hl(el, false));
        self.elem_nq.clear();   op.set_checked(false);  self.elem_op = None;
    }

    fn clear_state(&mut self) {     //log::info!("clear state");
        self.elem_nq.iter().for_each(|el| Self::toggle_hl(el, false));
        self.elem_nq.clear();   self.elem_op = None;    self.cnt = 1;

        let elem_eq = self.elem_eq.cast::<HtmlElement>().unwrap();
        elem_eq.class_list().remove_5("ring-red-400",   // XXX: better ideas?
            "text-red-500", "text-lime-500",
            "ring-lime-400", "ring-2").unwrap();
        elem_eq.set_inner_text("≠?");

        self.sol_div.cast::<HtmlElement>().unwrap().set_inner_text("");

        //let coll = web_sys::window().unwrap().document().unwrap()
        //    .get_element_by_id("num-operands").unwrap().children();
        let coll = self.num_div.cast::<HtmlElement>().unwrap().children();

        //let elem = elem.next_element_sibling().unwrap()   // XXX:
        //    .next_element_sibling().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        //if  elem.read_only() { elem.set_read_only(true); elem.blur().unwrap(); }

        for i in   0..coll.length() {
            let inp = coll.item(i).unwrap()
                .dyn_into::<HtmlInputElement>().unwrap();
            //if !inp.read_only() {   inp.blur().unwrap(); }
            inp.set_max_length(3);  inp.set_size(3);    inp.set_hidden(false);
        }
    }

    fn toggle_hl(el: &HtmlInputElement, hl: bool) {
        if hl {
            el.class_list().add_2("ring", "ring-purple-600").unwrap();
        } else {    // XXX:
            el.class_list().remove_2("ring", "ring-purple-600").unwrap();
        }
    }
}

enum Msg {
    Operator(HtmlInputElement),
    Operands(HtmlInputElement),
    Editable(HtmlInputElement),
    Update(u8, i32),
    Resize(u8),
    Restore,
    Resolve,
}

impl Component for Game24 {
    type Properties = ();
    type Message = Msg;

    fn create(_ctx: &Context<Self>) -> Self {
        let mut game = Self { goal: 24, nums: vec![],
            deck: (0..52).collect::<Vec<_>>(), pos: 0, cnt: 1,
            sol_div: NodeRef::default(), num_div: NodeRef::default(),
            elem_eq: NodeRef::default(), elem_op: None, elem_nq: VecDeque::new(),
        };  game.dealer(4);     game
    }

    //#[function_component(Game24F)] fn game24() -> Html
  fn view(&self, ctx: &Context<Self>) -> Html {
    let link = ctx.link();

    let ops_checked = link.callback(|e: Event|
        Msg::Operator(e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap()));
        // require value='xxx' in <input>, default is 'on'

    let cnt_changed = link.callback(|e: Event|
        Msg::Resize(e.target().unwrap()
            .dyn_into::<web_sys::HtmlSelectElement>().unwrap().value().parse::<u8>().unwrap()));

    let resolve = link.callback(|_| Msg::Resolve);
    let restore = link.callback(|_| Msg::Restore);
    let refresh = link.callback(|_| Msg::Resize(0));
    //web_sys::window().map(|win| win.location().reload());

    // XXX: drag to exchange/replace?

    let num_editable = link.callback(|e: MouseEvent|
        Msg::Editable(e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap()));

    let num_readonly = Callback::from(|e: FocusEvent| {
        let inp = e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        if  inp.read_only() { return }
        if  inp.check_validity() { inp.set_read_only(true); } else {
            inp.focus().unwrap();  inp.select();
        }
    });

    let num_changed = link.batch_callback(|e: Event| {
        let inp = e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        let str = inp.value();  log::info!("input {}", str);
        if !str.is_empty() && inp.check_validity() {    inp.set_read_only(true);
            Some(Msg::Update(inp.get_attribute("id").unwrap().get(1..).unwrap()
                .parse::<u8>().unwrap(), str.parse::<i32>().unwrap()))
        } else { inp.focus().unwrap();   inp.select();  None }
    });

    let num_checked = link.callback(|e: FocusEvent|
        Msg::Operands(e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap()));

    let num_class = "px-4 py-2 m-4 w-fit
        read-only:bg-transparent bg-stone-200 border border-purple-200
        text-center text-2xl text-purple-600 font-semibold
        hover:text-white hover:bg-purple-600 hover:border-transparent
        focus:outline-none focus:ring-2 focus:ring-purple-600 focus:ring-offset-2
        shadow-xl invalid:border-red-500 invalid:border-2";
    let nums = self.nums.iter().enumerate().map(|(idx, num)| {
        /*let (num, sid) = ((num % 13) + 1, (num / 13)/* % 4 */);

        let court = [ "T", "J", "Q", "K" ];
        let suits = [ "S", "C", "D", "H" ];     // "♣♦♥♠"
        let _ = format!(r"{}{}.svg", match num { 1 => "A".to_owned(),
            2..=9 => num.to_string(), 10..=13 => court[(num - 10) as usize].to_owned(),
            _ => "?".to_owned() }, suits[sid as usize]);     //num  // TODO: */

        html!{  // XXX: https://en.wikipedia.org/wiki/Playing_cards_in_Unicode
            <input type="text" value={ num.to_string() }
                id={ format!("N{idx}") } readonly=true draggable="true"
                placeholder="?" inputmode="numeric" pattern=r"-?\d+" maxlength="3" size="3"
                class={ classes!(num_class, "rounded-full") } data-bs-toggle="tooltip"
                title="Click to (un)check\nDouble click to input\nDrag over to exchange"/>
        }
    }).collect::<Html>();

    let ops = [ "+", "-", "×", "÷" ].into_iter().map(|op| html!{
        <div class="m-4 inline-block">
            <input type="radio" id={ op } value={ op } class="hidden peer"/>
            <label for={ op } draggable="true" data-bs-toggle="tooltip"
                title="Click to (un)check\nDrag over to replace/exchange"
                class="px-4 py-2 m-4 bg-indigo-600 text-white text-3xl font-bold
                hover:bg-indigo-400 peer-checked:outline-none peer-checked:ring-2
                peer-checked:ring-indigo-500 peer-checked:ring-offset-2
                peer-checked:bg-transparent rounded-md shadow-xl">{ op }</label>
        </div>
    }).collect::<Html>();

    let ctrl_class = "px-4 py-2 m-4 text-gray-900 font-bold bg-gradient-to-r
        from-stone-200 via-stone-400 to-stone-500 rounded-lg hover:bg-gradient-to-br
        focus:ring-4 focus:outline-none focus:ring-stone-300 shadow-lg shadow-stone-500/50
        dark:focus:ring-stone-800 dark:shadow-lg dark:shadow-stone-800/80";

    let cnt_options = (4..=6).map(|n| html!{
        <option value={ n.to_string() } selected={ n == self.nums.len() }>{
            format!("{n} nums") }</option>
    }).collect::<Html>();

    html!{ <main class="mt-auto mb-auto">
        //<div id="play-cards"/>    // TODO:

        <p class="hidden">{
            "Click on a operator and two numbers to form expression, " }<br/>{
            "repeat the process until all numbers are consumed, " }<br/>{
            "the final expression will be determined automatically." }<br/><br/></p>

        <div id="ops-group" onchange={ ops_checked }>{ ops }</div>

        <div id="expr-skel">
            /*<style>{ r"
                [contenteditable='true'].single-line { white-space: nowrap; overflow: hidden; }
                [contenteditable='true'].single-line br { display: none; }
                [contenteditable='true'].single-line  * { display: inline; white-space: nowrap; }
            " }</style>*/

            <div id="num-operands" class="inline-block" ref={ self.num_div.clone() }
                ondblclick={ num_editable.clone() } onchange={ num_changed.clone() }
                onfocus={ num_checked } onblur={ num_readonly.clone() }>{ nums }</div>

            // data-bs-toggle="collapse" data-bs-target="#all-solutions" aria-expanded="false" aria-controls="all-solutions"
            <button onclick={ resolve } ref={ self.elem_eq.clone() } //text-white
                class="py-2 px-4 m-4 text-3xl font-bold rounded-md
                hover:outline-none hover:ring-2 hover:ring-indigo-400
                focus:ring-indigo-500 focus:ring-offset-2"
                data-bs-toggle="tooltip" title="Click to get solutions">{ "≠?" }</button>
            <input type="text" value={ self.goal.to_string() }
                id={ format!("G{}", self.nums.len()) } readonly=true
                ondblclick={ num_editable } onchange={ num_changed } onblur={ num_readonly }
                placeholder="??" inputmode="numeric" pattern=r"-?\d+" maxlength="4" size="4"
                class={ classes!(num_class, "rounded-md") }
                data-bs-toggle="tooltip" title="Double click to input new goal"/>
        </div>

        <p class="hidden peer-invalid:visible relative -top-[1rem] text-red-500 font-light">{
             "Invalid integer number input, please correct it!" }</p> // invisible vs hidden

        <div id="ctrl-btns">
            <input type="reset" value={ "Restore" } class={ classes!(ctrl_class) }
                onclick={ restore } data-bs-toogle="tooltip" title="Click reset to initial"/>
            <select class={ classes!(ctrl_class) } onchange={ cnt_changed }
                data-bs-toogle="tooltip" title="Click to select numbers count">{
                cnt_options }</select>
            <button class={ classes!(ctrl_class) } onclick={ refresh }
                data-bs-toogle="tooltip" title="Click to refresh new">{ "Refresh" }</button>
        </div>

        <div id="all-solutions" ref={ self.sol_div.clone() }
            class="overflow-y-auto ml-auto mr-auto w-fit text-left text-lime-500 text-xl"
            data-bs-toggle="tooltip" title="All independent solutions"></div>
    </main> }
  }

    fn update  (&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Operator(inp) => { self.elem_op = Some(inp);
                if  self.elem_nq.len() == 2 { self.form_expr(); }   false
            }

            Msg::Operands(inp) => {
                let nq = &mut self.elem_nq;
                let mut n = nq.len();
                if  nq.iter().enumerate().any(|(i, el)| {
                    let same = el.is_same_node(Some(inp.as_ref()));
                    if  same { n = i; }     same }) {
                    Self::toggle_hl(&inp, false);
                    if n < nq.len() { nq.remove(n); }
                } else {
                    Self::toggle_hl(&inp, true);
                    nq.push_back(inp);
                }

                if 2 < nq.len() { Self::toggle_hl(&nq.pop_front().unwrap(), false); }
                if  nq.len() == 2 && self.elem_op != None { self.form_expr(); }     false
            }

            Msg::Editable(inp) => {
                //inp.set_selection_range(end, inp.value().len() as u32).unwrap();
                if self.cnt < 2 { inp.set_read_only(false); }
                if inp.get_attribute("id").unwrap().starts_with('N') {
                    self.update(_ctx, Msg::Operands(inp));  // don't check on editing
                }   false
            }

            Msg::Resize(n) => {    self.clear_state();
                if 0 < n { self.dealer(n as usize); } else {
                           self.dealer(self.nums.len());
                }   true
            }

            Msg::Restore => { self.clear_state();   true }
            Msg::Update(idx, val) => {  let idx = idx as usize;
                if idx == self.nums.len() { self.goal = val; } else {
                    self.nums[idx] = val;
                }   false
            }

            Msg::Resolve => {
                let sol = calc24_coll(&self.goal.into(),
                    &self.nums.iter().map(|&n|
                    Rational::from(n)).collect::<Vec<_>>(), DynProg);
                let cnt = sol.len();

                let mut sol = sol.into_iter().map(|str| {
                    let mut str = str.chars().map(|ch|
                        match ch { '*' => '×', '/' => '÷', _ => ch }).collect::<String>();
                    str.push_str("<br/>");  str
                }).collect::<Vec<String>>().concat();

                if 10 < cnt { sol.push_str(&format!(
                    "<br/>{cnt} solutions in total<br/>")); }
                self.sol_div.cast::<HtmlElement>().unwrap().set_inner_html(&sol);   false
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
            <style>{ r"body { text-align: center; height: 100vh; }" }</style>
                    // display: flex; flex-direction: column;

            <header><br/><h1 class="text-4xl"><a href="https://github.com/mhfan/inrust">{
                "24 Game/Puzzle/Challenge" }</a></h1><br/>
            </header>

            <Game24 />

            <footer><br/><p>{ "Copyright © 2022, " }    // &copy;
                <a href="https://github.com/mhfan">{ "mhfan" }</a></p><br/></footer>
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
