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
use instant::Instant;

struct Game24 {
    goal: Rational,
    nums: Vec<Rational>,

    deck: Vec<i32>, // hold all cards number
    spos: usize,    // shuffle position

    ncnt: usize,
    tnow: Instant,

    sol_elm: NodeRef,
    grp_elm: NodeRef,
    eqm_elm: NodeRef,

    opr_elm:   Option<HtmlInputElement>,
    opd_elq: VecDeque<HtmlInputElement>,
}

impl Game24 {
    fn new() -> Self {
        let mut game = Self { goal: 24.into(), nums: vec![],
            deck: (0..52).collect(), spos: 0, ncnt: 1, tnow: Instant::now(),
            sol_elm: NodeRef::default(), grp_elm: NodeRef::default(),
            eqm_elm: NodeRef::default(), opr_elm: None, opd_elq: VecDeque::new(),
        };  game.dealer(4);     game
    }

    fn dealer(&mut self, n: usize) {
        use rand::{thread_rng, seq::SliceRandom};
        let mut rng = thread_rng();

        loop {  if self.spos == 0 { self.deck.shuffle(&mut rng); }
            self.nums = self.deck[self.spos..].partial_shuffle(&mut rng,
                n).0.iter().map(|n| Rational::from((n % 13) + 1)).collect();
            self.spos += n; if self.deck.len() < self.spos + n { self.spos = 0; }

            if !calc24_first(&self.goal, &self.nums, DynProg).is_empty() { break }
        }   self.tnow = Instant::now();
    }

    fn form_expr(&mut self) {
        let opr = self.opr_elm.as_ref().unwrap();
        let opd = &mut self.opd_elq;
        let str = format!("({} {} {})", opd[0].value(), opr.value(), opd[1].value());

        opd[1].set_size (str.len() as u32);     opd[1].set_max_length(str.len() as i32);
        opd[1].set_value(&str);     opd[1].blur().unwrap();     opd[0].set_hidden(true);

        self.ncnt += 1; if self.ncnt == self.nums.len() {
            let str = str.chars().map(|ch|
                match ch { '×' => '*', '÷' => '/', _ => ch }).collect::<String>();
            let eqm_elm = self.eqm_elm.cast::<HtmlElement>().unwrap();

            if Rational::from((mexe::eval(str).unwrap()
                + 0.1) as i32) == self.goal {
                let dur = self.tnow.elapsed();  self.tnow = Instant::now();
                log::info!("timing: {:.1}s", dur.as_secs_f32());    // TODO: show it on page

                eqm_elm.class_list().add_3("ring-2", "text-lime-500",
                    "ring-lime-400").unwrap();
                eqm_elm.set_inner_text("=");
            } else {    // XXX:
                eqm_elm.class_list().add_3("ring-2", "text-red-500",
                    "ring-red-400").unwrap();
                eqm_elm.set_inner_text("≠");
            }
        }

        opd.iter().for_each(|el| Self::toggle_hl(el, false));
        opd.clear();    opr.set_checked(false);     self.opr_elm = None;
    }

    fn clear_state(&mut self) {     //log::info!("clear state");
        self.opd_elq.iter().for_each(|el| Self::toggle_hl(el, false));
        self.opd_elq.clear();   self.opr_elm = None;    self.ncnt = 1;

        let  eqm_elm = self.eqm_elm.cast::<HtmlElement>().unwrap();
        eqm_elm.class_list().remove_5("ring-red-400",   // XXX: better ideas?
            "text-red-500", "text-lime-500",
            "ring-lime-400", "ring-2").unwrap();
        eqm_elm.set_inner_text("≠?");

        self.sol_elm.cast::<HtmlElement>().unwrap().set_inner_text("");
        let coll = self.grp_elm.cast::<HtmlElement>().unwrap().children();
        //let coll = web_sys::window().unwrap().document().unwrap()
        //    .get_element_by_id("nums-group").unwrap().children();

        for i in 0..coll.length() {
            let inp = coll.item(i).unwrap()
                .dyn_into::<HtmlInputElement>().unwrap();
            if (self.nums.len() as u32 - 1) < i { inp.set_hidden(true); continue }
            inp.set_max_length(3);  inp.set_size(3);    inp.set_hidden(false);
        }   //log::info!("clear state");
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
    Update(Option<u8>, Rational),
    Resize(u8),
    Restore,
    Resolve,
}

impl Component for Game24 {
    type Properties = ();
    type Message = Msg;

    fn create(_ctx: &Context<Self>) -> Self { Self::new() }

    fn update  (&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Operator(inp) => { self.opr_elm = Some(inp);
                if  self.opd_elq.len() == 2 { self.form_expr(); }   false
            }

            Msg::Operands(inp) => {
                if  self.ncnt == self.nums.len() { return false }
                let opd = &mut self.opd_elq;
                let mut idx = opd.len();

                if  opd.iter().enumerate().any(|(i, el)| {
                        let dup = el.is_same_node(Some(inp.as_ref()));
                        if  dup { idx = i; }  dup }) {
                    if  idx < opd.len() { opd.remove(idx); }
                            Self::toggle_hl(&inp, false);
                } else {    Self::toggle_hl(&inp, true);

                    if  opd.len() == 2 {
                        Self::toggle_hl(&opd.pop_front().unwrap(), false);
                    }   opd.push_back(inp);
                    if  opd.len() == 2 && self.opr_elm != None { self.form_expr(); }
                }   false
            }

            Msg::Editable(inp) => if 1 < self.ncnt { false } else {
                /* && self.ncnt < self.nums.len() &&
                inp.get_attribute("id").unwrap().starts_with("N")*/
                //let end = inp.value().len() as u32;
                //inp.set_selection_range(end, end).unwrap();
                inp.set_read_only(false);   true
            }

            Msg::Resize(n) => if 9 < n {
                log::error!("too big to support: {}", n);   false
            } else {
                self.dealer(if 0 < n { n as usize } else { self.nums.len() });
                self.clear_state();     true
            }

            Msg::Restore => { self.clear_state();   true }
            Msg::Update(idx, val) => {
                if let Some(idx) = idx {    let idx = idx as usize;
                    if  idx < self.nums.len() { self.nums[idx] = val; } else {
                        log::error!("index overflow: {}", idx);
                    }
                } else { self.goal = val; }     false
            }

            Msg::Resolve => {
                let sols = calc24_coll(&self.goal, &self.nums, DynProg);
                let cnt = sols.len();

                let mut sols = sols.into_iter().map(|str| {
                    let mut str = str.chars().map(|ch| match ch {
                        '*' => '×', '/' => '÷', _ => ch }).collect::<String>();
                    str.push_str("<br/>");  str
                }).collect::<Vec<_>>().concat();

                if 5 < cnt {
                    sols.push_str(&format!("<br/>{cnt} solutions in total<br/>"));
                }   self.sol_elm.cast::<HtmlElement>().unwrap().set_inner_html(&sols);  false
            }
        }
    }

    //fn changed (&mut self, ctx: &Context<Self>) -> bool { true }
    //fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {}
    //fn destroy (&mut self, ctx: &Context<Self>) {}

  fn view(&self, ctx: &Context<Self>) -> Html {
    let link = ctx.link();  // Callback::from()
    //web_sys::window().map(|win| win.location().reload());
    // XXX: drag to exchange/replace?

    let cnt_changed = link.batch_callback(|e: Event|
        e.target().and_then(|t|
            t.dyn_into::<web_sys::HtmlSelectElement>().ok().and_then(|sel|
                sel.value().parse::<u8>().ok().map(Msg::Resize))));

    let ops_checked = link.batch_callback(|e: Event|
        e.target().unwrap().dyn_into::<HtmlInputElement>().ok().map(Msg::Operator));

    let num_checked = link.batch_callback(|e: MouseEvent|
        e.target().unwrap().dyn_into::<HtmlInputElement>().ok().map(Msg::Operands));

    let num_editable = link.batch_callback(|e: MouseEvent|
        e.target().unwrap().dyn_into::<HtmlInputElement>().ok().map(Msg::Editable));

    let num_changed = link.batch_callback(|e: FocusEvent| {
        let inp = e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        if  inp.read_only() { return None }

        if  inp.check_validity() {   inp.set_read_only(true);
            Some(Msg::Update(inp.get_attribute("id").unwrap().get(1..).unwrap()
                .parse::<u8>().ok(), inp.value().parse::<Rational>().unwrap()))
        } else { inp.focus().unwrap();   inp.select();  None }
    });

    let num_class = "px-4 py-2 my-4 w-fit appearance-none select-text
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

        html! { // https://en.wikipedia.org/wiki/Playing_cards_in_Unicode
            <input type="text" value={ num.to_string() } id={ format!("N{idx}") }
                maxlength="3" size="3" readonly=true draggable="true"
                placeholder="?" inputmode="numeric" pattern=r"-?\d+(\/\d+)?"
                class={ classes!(num_class, "rounded-full", "mx-2") }/>
        }       // https://regexr.com
    }).collect::<Html>();

    let ctrl_class = "px-4 py-2 m-4 text-gray-900 font-bold bg-gradient-to-r
        from-stone-200 via-stone-400 to-stone-500 rounded-lg hover:bg-gradient-to-br
        focus:ring-4 focus:outline-none focus:ring-stone-300 shadow-lg shadow-stone-500/50
        dark:focus:ring-stone-800 dark:shadow-lg dark:shadow-stone-800/80";

    html! { <main class="mt-auto mb-auto">
        <div id="play-cards"/>    // TODO:

        <p class="hidden">{
            "Click on a operator and two numbers to form expression, " }<br/>{
            "repeat the process until all numbers are consumed, " }<br/>{
            "the final expression will be determined automatically." }<br/><br/></p>

        <div id="ops-group" onchange={ ops_checked } data-bs-toggle="tooltip"
            title="Click to (un)check\nDrag over to replace/exchange">{
            [ "+", "-", "×", "÷" ].into_iter().map(|op| html! {
                <div class="mx-6 my-4 inline-block">
                    <input type="radio" id={ op } value={ op } name="ops"
                        class="hidden peer"/>   // require value='xxx', default is 'on'

                    <label for={ op } draggable="true"
                        class="px-4 py-2 bg-indigo-600 text-white text-3xl font-bold
                        hover:bg-indigo-400 peer-checked:outline-none peer-checked:ring-2
                        peer-checked:ring-indigo-500 peer-checked:ring-offset-2
                        peer-checked:bg-transparent rounded-md shadow-xl">{ op }</label>
                </div>
            }).collect::<Html>()
        }</div>

        <div id="expr-skel">
            <span id="nums-group" ref={ self.grp_elm.clone() } data-bs-toggle="tooltip"
                title="Click to (un)check\nDouble click to input\nDrag over to exchange"
                ondblclick={ num_editable.clone() } onclick={ num_checked }
                onblur={ num_changed.clone() }>{ nums }</span>

            // data-bs-toggle="collapse" data-bs-target="#all-solutions"
            //       aria-expanded="false" aria-controls="all-solutions" //text-white
            <button ondblclick={ link.callback(|_| Msg::Resolve) } ref={ self.eqm_elm.clone() }
                class="px-4 py-2 m-4 text-3xl font-bold rounded-md hover:outline-none
                hover:ring-2 hover:ring-indigo-400 focus:ring-indigo-500 focus:ring-offset-2"
                data-bs-toggle="tooltip" title="Double click to get solutions">{ "≠?" }</button>

            <input type="text" id="G" value={ self.goal.to_string() } readonly=true
                ondblclick={ num_editable } onblur={ num_changed }
                placeholder="??" inputmode="numeric" pattern=r"-?\d+(\/\d+)?"
                maxlength="4" size="4" class={ classes!(num_class, "rounded-md") }
                data-bs-toggle="tooltip" title="Double click to input new goal"/>

            /*<style>{ r"
                [contenteditable='true'].single-line { white-space: nowrap; overflow: hidden; }
                [contenteditable='true'].single-line br { display: none; }
                [contenteditable='true'].single-line  * { display: inline; white-space: nowrap; }
            " }</style>*/
        </div>

        <p class="hidden peer-invalid:visible relative -top-[1rem] text-red-500 font-light">{
             "Invalid integer number input, please correct it!" }</p> // invisible vs hidden

        <div id="ctrl-btns">
            <input type="reset" value="Restore" class={ classes!(ctrl_class) }
                onclick={ link.callback(|_| Msg::Restore) }
                data-bs-toogle="tooltip" title="Click reset to initial"/>

            <select class={ classes!(ctrl_class, "appearance-none") } onchange={ cnt_changed }
                data-bs-toogle="tooltip" title="Click to select numbers count">{
                (4..=6).map(|n| html! {
                    <option value={ n.to_string() } selected={ n == self.nums.len() }>{
                        format!("{n} nums") }</option>
                }).collect::<Html>()
            }</select>
            <button class={ classes!(ctrl_class) } onclick={ link.callback(|_| Msg::Resize(0)) }
                data-bs-toogle="tooltip" title="Click to refresh new">{ "Refresh" }</button>
        </div>

        <div id="all-solutions" ref={ self.sol_elm.clone() }
            class="overflow-y-auto ml-auto mr-auto w-fit text-left text-lime-500 text-xl"
            data-bs-toggle="tooltip" title="All inequivalent solutions"></div>
    </main> }
  }
}

#[function_component(GHcorner)] fn gh_corner() -> Html { html! {
    <a href={ env!("CARGO_PKG_REPOSITORY") }
        class="github-corner" aria-label="View source on GitHub">
        <svg width="60" height="60" viewBox="0 0 250 250" aria-hidden="true"
            style="fill:#ddd; color:#151513; position: absolute; top: 0; border: 0; right: 0;">

            <path d="M0,0 L115,115 L130,115 L142,142 L250,250 L250,0 Z"/>
            <path d="M128.3,109.0 C113.8,99.7 119.0,89.6 119.0,89.6 C122.0,82.7 120.5,78.6
                120.5,78.6 C119.2,72.0 123.4,76.3 123.4,76.3 C127.3,80.9 125.5,87.3 125.5,87.3
                C122.9,97.6 130.6,101.9 134.4,103.2" style="transform-origin: 130px 106px;"
                fill="currentColor" class="octo-arm"/>
            <path d="M115.0,115.0 C114.9,115.1 118.7,116.5 119.8,115.4 L133.7,101.6 C136.9,99.2
                139.9,98.4 142.2,98.6 C133.8,88.0 127.5,74.4 143.8,58.0 C148.5,53.4 154.0,51.2
                159.7,51.0 C160.3,49.4 163.2,43.6 171.4,40.1 C171.4,40.1 176.1,42.5 178.8,56.2
                C183.1,58.6 187.2,61.8 190.9,65.4 C194.5,69.0 197.7,73.2 200.1,77.6 C213.8,80.2
                216.3,84.9 216.3,84.9 C212.7,93.1 206.9,96.0 205.4,96.6 C205.1,102.4 203.0,107.8
                198.3,112.5 C181.9,128.9 168.3,122.5 157.7,114.1 C157.9,116.9 156.7,120.9
                152.7,124.9 L141.0,136.5 C139.8,137.7 141.6,141.9 141.8,141.8 Z"
                fill="currentColor" class="octo-body"/>
        </svg>  // https://github.com/tholman/github-corners
        <style>{ ".github-corner:hover .octo-arm { animation: octocat-wave 560ms ease-in-out }
            @keyframes octocat-wave {
                0%,100% { transform: rotate(0) }
                20%,60% { transform: rotate(-25deg) }
                40%,80% { transform: rotate(10deg) }
            }
            @media (max-width: 500px) {
                  .github-corner:hover .octo-arm { animation: none }
                  .github-corner       .octo-arm { animation: octocat-wave 560ms ease-in-out }
            }"
        }</style>
    </a> }
}

fn root_route(routes: &RootRoute) -> Html {
    #[allow(clippy::let_unit_value)] match routes {
        RootRoute::Home  => html! { <>
            //margin: 0 auto;   //class: justify-center;    // XXX: not working
            <style>{ r"html { background-color: #15191D; color: #DCDCDC; }
                body { font-family: Courier, Monospace; text-align: center; height: 100vh; }"
            }</style>   // display: flex; flex-direction: column;

            <header><GHcorner/><h1 class="text-4xl m-4">
                <a href="https://github.com/mhfan/inrust">{ "'24' Challenge" }</a>
            </h1></header>

            <Game24/>
            // https://css-tricks.com
            // https://www.w3schools.com
            // https://developer.mozilla.org/en-US/docs/Web/HTML

            <footer><p clase="m-4">{ "Copyright © 2022 " }  // &copy; // "absolute bottom-0"
                <a href="https://github.com/mhfan">{ "mhfan" }</a>
                //<div align="center">
                //    <img src="https://page-views.glitch.me/badge?page_id=/24-puzzle"/></div>
            </p></footer>
        </> },

        RootRoute::Route => html! { <Switch<Route> render={Switch::render(switch)} /> },
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::About => html! { <p>{ "About" }</p> },
        Route::NotFound => html! { <p>{ "Not Found" }</p> },
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
        RootRoute::Home  => html! { <p class="text-4xl">{ "Yew Template" }</p> },
        RootRoute::About => html! { <p>{ "About" }</p> },
        RootRoute::NotFound => html! { <p>{ "Not Found" }</p> },
    }
}

// =================================================================================== */

#[function_component(App)] fn app() -> Html {   // main root
    html! {
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
