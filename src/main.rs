
use yew::prelude::*;
use yew_router::prelude::*;

use std::collections::VecDeque;
use web_sys::{HtmlElement, HtmlInputElement, HtmlFieldSetElement};
use wasm_bindgen::JsCast;
use inrust::calc24::*;
use instant::Instant;

struct Game24State {
    goal: Rational,
    nums: Vec<Rational>,

    deck: Vec<u8>,  // hold all cards number
    spos: u8,       // shuffle position

    ncnt: u8,       // combined numbers count
    tnow: Instant,  // timing

    tmr_elm: NodeRef,   // XXX: use reactivity instead, maybe all these NodeRef
    sol_elm: NodeRef,
    eqm_elm: NodeRef,
    ovr_elm: NodeRef,
    grp_opd: NodeRef,
    grp_opr: NodeRef,

    opr_elm:   Option<HtmlInputElement>,    // element of selected operator
    opd_elq: VecDeque<HtmlInputElement>,    // elements queue of selected operands
}

impl Game24State {
    fn new() -> Self {
        let mut game24 = Self {  goal: 24.into(), nums: vec![],
            deck: (0..52).collect(), spos: 0, ncnt: 1, tnow: Instant::now(),
            sol_elm: NodeRef::default(), eqm_elm: NodeRef::default(),
            grp_opd: NodeRef::default(), grp_opr: NodeRef::default(),
            tmr_elm: NodeRef::default(), ovr_elm: NodeRef::default(),
            opr_elm: None, opd_elq: VecDeque::new(),
        };  game24.dealer(4);   game24
    }

    fn dealer(&mut self, cnt: u8) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        //let dst = distributions::Uniform::new(1, 100);
        let cnt = if 0 < cnt { cnt } else { self.nums.len() as u8 };

        loop {
            if self.deck.len() < (self.spos + cnt) as usize { self.spos = 0; }
            if self.spos == 0 {   self.deck.shuffle(&mut rng); }

            self.nums = self.deck[self.spos as usize..]
                .partial_shuffle(&mut rng, cnt as usize).0.iter().map(|&n|
                    Rational::from((n as i32 % 13) + 1)).collect();     self.spos += cnt;
            //self.nums = (&mut rng).sample_iter(dst).take(4).map(Rational::from).collect();

            if !calc24_first(&self.goal, &self.nums, DynProg).is_empty() { break }
        }   self.tnow = Instant::now();
    }

    fn form_expr(&mut self) {
        let opd = &self.opd_elq;
        let opr = self.opr_elm.as_ref().unwrap();

        let str = format!("({} {} {})", opd[0].value(), opr.value(), opd[1].value());
        opd[0].set_size(str.len() as u32);  opd[0].set_value(&str);
        opd.iter().for_each(|elm| set_checked(elm, false));
        opd[1].set_hidden(true);    opr.set_checked(false);

        self.opd_elq.clear();       self.opr_elm = None;
        self.ncnt += 1;     if self.ncnt == self.nums.len() as u8 {
            let str = str.chars().map(|ch|
                match ch { '×' => '*', '÷' => '/', _ => ch }).collect::<String>();

            //opr.parent_element().unwrap().parent_element().unwrap()
            //    .dyn_into::<HtmlFieldSetElement>().unwrap().set_disabled(true);
            self.grp_opr.cast::<HtmlFieldSetElement>().unwrap().set_disabled(true);
            let eqm_elm = &self.eqm_elm.cast::<HtmlElement>().unwrap();

            if str.parse::<Expr>().unwrap().value() == &self.goal {
                let tmr_elm = &self.tmr_elm.cast::<HtmlElement>().unwrap();
                tmr_elm.set_inner_text(&format!("{:.1}s", self.tnow.elapsed().as_secs_f32()));
                tmr_elm.set_hidden(false);

                        eqm_elm.set_inner_text("=");    set_checked(eqm_elm, true);
            } else {    eqm_elm.set_inner_text("≠");
                eqm_elm.set_attribute("aria-checked", "false").unwrap();
            }
        }
    }

    fn clear_state(&mut self) {     //log::debug!("clear state");
        self.opd_elq.iter().for_each(|elm| set_checked(elm, false));
        if let Some(opr) = &self.opr_elm { opr.set_checked(false); }
        self.opd_elq.clear();   self.opr_elm = None;    self.ncnt = 1;

        let  eqm_elm = &self.eqm_elm.cast::<HtmlElement>().unwrap();
        eqm_elm.set_inner_text("≠?");   set_checked(eqm_elm, false);     // XXX: "mixed"
        self.tmr_elm.cast::<HtmlElement>().unwrap().set_hidden(true);
        self.sol_elm.cast::<HtmlElement>().unwrap().set_hidden(true);

        let ovr_elm = &self.ovr_elm.cast::<HtmlInputElement>().unwrap();
        if !ovr_elm.hidden() { ovr_elm.set_value(""); }
        self.grp_opr.cast::<HtmlFieldSetElement>().unwrap().set_disabled(!ovr_elm.hidden());

        let coll = self.grp_opd.cast::<HtmlElement>().unwrap().children();
        //let coll = web_sys::window().unwrap().document().unwrap()
        //    .get_element_by_id("nums-group").unwrap().children();

        for i in 0..coll.length() {
            let inp = coll.item(i).unwrap()
                .dyn_into::<HtmlInputElement>().unwrap();
            inp.set_size(3);    inp.set_hidden(false);
        }
    }
}

fn set_checked(elm: &HtmlElement, checked: bool) {
    if checked { elm.   set_attribute("aria-checked", "true").unwrap();
    } else {     elm.remove_attribute("aria-checked").unwrap(); }
}

enum Msg {
    Operator(HtmlInputElement),
    Operands(HtmlInputElement),
    Editable(HtmlInputElement),
    Update(Option<u8>, Rational),   // Input
    Overall(String),
    Resize(u8),
    Dismiss,
    Resolve,
}

impl Component for Game24State {
    type Properties = ();
    type Message = Msg;

    fn create(_ctx: &Context<Self>) -> Self { Self::new() }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Operator(inp) => { //log::debug!("{}", inp.checked());
                if  inp.is_same_node(self.opr_elm.as_ref().map(|elm|
                    elm.as_ref())) { self.opr_elm = None;
                    inp.set_checked(false);     return false;
                }                    self.opr_elm = Some(inp);
                if  self.opd_elq.len() == 2 { self.form_expr(); }
            }

            Msg::Operands(inp) => if self.ncnt != self.nums.len() as u8 {
                let opd = &mut self.opd_elq;
                let mut idx = opd.len();

                if  opd.iter().enumerate().any(|(i, elm)|
                    if elm.is_same_node(Some(inp.as_ref())) { idx = i; true } else { false }) {
                    opd.remove(idx);    inp.blur().unwrap();
                         set_checked(&inp, false);
                } else { set_checked(&inp, true);

                    if 1 < idx { set_checked(&opd.pop_front().unwrap(), false);
                    }   opd.push_back(inp);
                    if 0 < idx && self.opr_elm.is_some() { self.form_expr(); }
                }
            }

            Msg::Editable(inp) => if 1 == self.ncnt {
                /* && self.ncnt < self.nums.len() &&
                    inp.get_attribute("id").unwrap().starts_with("N")*/
                //let end = inp.value().len() as u32;
                //inp.set_selection_range(end, end).unwrap();
                inp.set_read_only(false);   inp.focus().unwrap();
            }

            Msg::Overall(s) => {
                self.nums = s.split_ascii_whitespace()
                    .filter_map(|s| s.parse::<Rational>().ok()).collect();
                self.sol_elm.cast::<HtmlElement>().unwrap().set_hidden(true);
                //yew::Component::update(self, _ctx, Msg::Resolve);
            }

            Msg::Resize(cnt) => {     debug_assert!(cnt < 10, "too big to solve!");
                let grp_opd = &self.grp_opd.cast::<HtmlElement>().unwrap();
                let ovr_elm = &self.ovr_elm.cast::<HtmlElement>().unwrap();

                let ovr = ovr_elm.hidden();
                if 1 == cnt {                               self.nums.clear();
                              grp_opd.set_hidden(true);     ovr_elm.set_hidden(false);
                } else if 0 != cnt || ovr {                 self.dealer(cnt);
                    if !ovr { grp_opd.set_hidden(false);    ovr_elm.set_hidden(true); }
                }   self.clear_state();     return true
            }

            Msg::Dismiss => { self.clear_state();   return true }
            Msg::Update(idx, val) => {  self.tnow = Instant::now();
                self.tmr_elm.cast::<HtmlElement>().unwrap().set_hidden(true);
                self.sol_elm.cast::<HtmlElement>().unwrap().set_hidden(true);

                if let Some(idx) = idx { self.nums[idx as usize] = val;
                    debug_assert!(idx < self.nums.len() as u8, "index overflow");
                } else { self.goal = val; }
            }

            Msg::Resolve => {
                if  self.nums.is_empty() {  // !ovr_elm.hidden() && ovr_elm.value().is_empty()
                    self.ovr_elm.cast::<HtmlInputElement>().unwrap().focus().unwrap();
                    return false
                }

                let sols = calc24_coll(&self.goal, &self.nums, DynProg);
                let cnt  = sols.len();

                let sols = sols.into_iter().map(|str| str.chars().map(|ch|
                        match ch { '*' => '×', '/' => '÷', _ => ch })
                    .chain("<br/>".chars()).collect())
                    .chain(std::iter::once_with(|| if 5 < cnt { format!(
                        "<span class=\"text-white\">Got {cnt} solutions</span><br/>")
                    } else if cnt < 1 { String::from(
                        "<span class=\"text-red-500\">Got NO solutions!</span><br/>")
                    } else { String::new() })).collect::<String>();

                let sol_elm = &self.sol_elm.cast::<HtmlElement>().unwrap();
                sol_elm.set_inner_html(&sols);  sol_elm.set_hidden(false);
            }
        }   false
    }

    //fn changed (&mut self, ctx: &Context<Self>) -> bool { true }
    //fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {}
    //fn destroy (&mut self, ctx: &Context<Self>) {}

  fn view(&self, ctx: &Context<Self>) -> Html {
    let link = ctx.link();  // Callback::from(|_| {})
    //web_sys::window().map(|win| win.location().reload());
    // XXX: drag to exchange/replace?

    let num_editable = link.batch_callback(|e: MouseEvent|
        e.target().and_then(|t| t.dyn_into().ok().map(Msg::Editable)));
        //e.prevent_default();  // prevent dblclick from selection?

    let num_focusout = Callback::from(|e: FocusEvent| {
        let inp = e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        if !inp.read_only() { inp.set_read_only(true); }
    });

    let num_changed = link.batch_callback(|e: Event| {
        let inp = e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();

        if  inp.check_validity() {   inp.set_read_only(true);
            Some(Msg::Update(inp.get_attribute("id").unwrap().get(1..).unwrap()
                .parse::<u8>().ok(), inp.value().parse::<Rational>().unwrap()))
        } else { if inp.focus().is_ok() { inp.select() }    None }
    });

    let num_class = "px-4 py-2 my-4 w-fit appearance-none select-text
        read-only:bg-transparent bg-stone-200 border border-purple-200
        text-center text-2xl text-purple-600 font-semibold
        hover:text-white hover:bg-purple-600 hover:border-transparent
        focus:outline-none focus:ring-2 focus:ring-purple-600 focus:ring-offset-2
        shadow-xl invalid:border-red-500 invalid:border-2";

    let nums = self.nums.iter().enumerate().map(|(idx, num)| {
        /*let (num, sid) = ((num % 13) + 1, (num / 13)/* % 4 */);
        // https://en.wikipedia.org/wiki/Playing_cards_in_Unicode

        let court = [ "T", "J", "Q", "K" ];
        let suits = [ "S", "C", "D", "H" ];     // "♣♦♥♠"
        let _ = format!(r"{}{}.svg", match num { 1 => "A".to_owned(),
            2..=9 => num.to_string(), 10..=13 => court[(num - 10) as usize].to_owned(),
            _ => "?".to_owned() }, suits[sid as usize]);     //num  // TODO: */

        html! { <input type="text" id={ format!("N{idx}") } value={ num.to_string() }
            maxlength="6" size="3" readonly=true name="nums" draggable="true"
            placeholder="?" inputmode="numeric" pattern=r"-?\d+(\/\d+)?"
            class={ classes!(num_class, "aria-checked:ring-purple-600",
                "aria-checked:ring", "rounded-full", "mx-2") }/>
        }   // https://regexr.com, https://regex101.com
    }).collect::<Html>();

    let ctrl_class = "px-4 py-2 m-4 text-gray-900 font-bold bg-gradient-to-r
        from-stone-200 via-stone-400 to-stone-500 rounded-lg hover:bg-gradient-to-br
        focus:ring-4 focus:outline-none focus:ring-stone-300 shadow-lg shadow-stone-500/50
        dark:focus:ring-stone-800 dark:shadow-lg dark:shadow-stone-800/80";

    //let resolving = use_state_eq(|| false);     // XXX: reactive
    html! { <main class="mt-auto mb-auto">
        <div id="play-cards"/>    // TODO:

        <p class="hidden">{
            "Click on a operator and two numbers to form expression, " }<br/>{
            "repeat the process until all numbers are consumed, " }<br/>{
            "the final expression will be determined automatically." }<br/><br/></p>

        <fieldset id="ops-group" ref={ &self.grp_opr }
            // use onclick instead of onchange for capable of de-selection
            onclick={ link.batch_callback(|e: MouseEvent| e.target().and_then(|t|
                t.dyn_into().ok().map(Msg::Operator))) } data-bs-toggle="tooltip"
            title="Click to (un)check\nDrag over to replace/exchange">{

            [ "+", "-", "×", "÷" ].into_iter().map(|op| html! {
                <div class="mx-6 my-4 inline-block">
                    <input type="radio" name="ops" id={ op } value={ op }
                        class="hidden peer"/>   // require value='xxx', default is 'on'

                    <label for={ op } draggable="true"
                        class="px-4 py-2 bg-indigo-600 text-white text-3xl font-bold
                        hover:bg-indigo-400 peer-checked:outline-none peer-checked:ring-2
                        peer-checked:ring-indigo-500 peer-checked:ring-offset-2
                        peer-checked:bg-transparent rounded-md shadow-xl">{ op }</label>
                </div>
            }).collect::<Html>()
        }</fieldset>

        <div id="expr-skel">
            <span id="nums-group" ref={ &self.grp_opd } data-bs-toggle="tooltip"
                title="Click to (un)check\nDouble click to input\nDrag over to exchange"
                onfocusout={ &num_focusout } ondblclick={ &num_editable }
                onchange={ &num_changed } onclick={ link.batch_callback(|e: MouseEvent|
                    e.target().and_then(|t| t.dyn_into().ok().map(Msg::Operands))) }
            >{ nums }</span>

            <input type="text" id="overall" name="operands" //minlength="32" size="16"
                data-bs-toggle="tooltip" title="Input space seperated numbers"
                placeholder="???" inputmode="numeric" pattern=r"\s*(-?\d+(\/\d+)?\s*){2,9}"
                onchange={ link.batch_callback(|e: Event| {
                    let inp = e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
                    let vs  = inp.value();
                    if  inp.check_validity() && !vs.is_empty() { Some(Msg::Overall(vs))
                    } else { if inp.focus().is_ok() { inp.select() }    None }
                })} ref={ &self.ovr_elm } hidden=true
                class={ classes!(num_class, "aria-checked:ring-purple-600",
                    "aria-checked:ring", "rounded-full", "mx-2") }/>

            // data-bs-toggle="collapse" data-bs-target="#all-sols"
            //       aria-expanded="false" aria-controls="all-sols"
            <button ondblclick={ link.callback(|_| Msg::Resolve) } ref={ &self.eqm_elm }
                class="px-4 py-2 m-4 text-3xl font-bold rounded-md aria-[checked=false]:ring-2
                aria-checked:ring-2 aria-checked:text-lime-500 aria-checked:ring-lime-400
                aria-[checked=false]:text-red-500 aria-[checked=false]:ring-red-400
                hover:outline-none hover:ring-2 hover:ring-indigo-400
                focus:ring-indigo-500 focus:ring-offset-2" //text-white
                data-bs-toggle="tooltip" title="Double click to get solutions">{ "≠?" }</button>

            <input type="text" id="G" value={ self.goal.to_string() } readonly=true
                onfocusout={ num_focusout } ondblclick={ num_editable } onchange={ num_changed }
                placeholder="??" inputmode="numeric" pattern=r"-?\d+(\/\d+)?"
                maxlength="8" size="4" class={ classes!(num_class, "rounded-md") }
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
            <input type="reset" value="Dismiss" class={ classes!(ctrl_class) }
                onclick={ link.callback(|_| Msg::Dismiss) }
                data-bs-toogle="tooltip" title="Click to dismiss expr."/>

            <select class={ classes!(ctrl_class, "appearance-none") } data-bs-toogle="tooltip"
                onchange={ link.batch_callback(|e: Event| e.target().and_then(|t|
                    t.dyn_into::<web_sys::HtmlSelectElement>() .ok().and_then(|sel|
                        sel.value().parse::<u8>().ok().map(Msg::Resize)))) }
                title="Click to set numbers count\nOverall - single element for all numbers">

                <option value="1">{ "Overall" }</option>{
                (4..=6).map(|n| html! { <option value={ n.to_string() }
                    selected={ self.nums.len() == n }>{ format!("{n} nums") }</option>
                }).collect::<Html>()
            }</select>

            <button class={ classes!(ctrl_class) } onclick={ link.callback(|_| Msg::Resize(0)) }
                data-bs-toogle="tooltip" title="Click to refresh new">{ "Refresh" }</button>
        </div>

        <div id="timer" ref={ &self.tmr_elm } hidden=true
            data-bs-toggle="tooltip" title="Time for calculation"
            class="mx-1 font-sans text-yellow-600 absolute left-0"></div>

        <div id="all-sols" ref={ &self.sol_elm } hidden=true
            class="overflow-y-auto ml-auto mr-auto w-fit text-left text-lime-500 text-xl"
            data-bs-toggle="tooltip" title="All inequivalent solutions"></div>
    </main> }
  }
}

fn main_route(routes: MainRoute) -> Html {
    #[function_component(GHcorner)] fn gh_corner() -> Html {
        /*let elm = web_sys::window().unwrap().document().unwrap()
            .create_element("a").unwrap();
        elm.set_class_name("github-corner");
        elm.set_attribute("href",  env!("CARGO_PKG_REPOSITORY")).unwrap();
        elm.set_attribute("aria-label", "View source on GitHub").unwrap();
        elm.set_inner_html(include_str!("../assets/gh-corner.html"));
        Html::VRef(elm.into()) */

        html! { <a href={ env!("CARGO_PKG_REPOSITORY") }
            class="github-corner" aria-label="View source on GitHub">{
            Html::from_html_unchecked(include_str!("../assets/gh-corner.html").into())
        }</a> }
    }

    #[allow(clippy::let_unit_value)] match routes {
        MainRoute::Home  => html! { <>
            //margin: 0 auto;   //class: justify-center;    // XXX: not working
            <style>{ r"html { background-color: #15191D; color: #DCDCDC; }
                body { font-family: Courier, Monospace; text-align: center; height: 100vh; }"
            }</style>   // display: flex; flex-direction: column;

            <header class="text-4xl m-8"> <GHcorner/>
                //{ Html::from_html_unchecked(include_str!("../assets/gh-corner.html").into()) }
                <a href="https://github.com/mhfan/inrust">{ "24 Challenge" }</a>
            </header>

            <Game24State/>
            // https://css-tricks.com
            // https://www.w3schools.com
            // https://developer.mozilla.org/en-US/docs/Web/HTML

            <footer clase="m-4">{ "Copyright © 2022 by " }  // &copy; // "absolute bottom-0"
                <a href="https://github.com/mhfan">{ "mhfan" }</a>
                //<div align="center">
                //    <img src="https://page-views.glitch.me/badge?page_id=/24-puzzle"/></div>
            </footer>
        </> },

        MainRoute::Subs => html! { <Switch<SubRoute> render={ switch }/> },
    }
}

fn switch(routes: SubRoute) -> Html {
    match routes {
        SubRoute::About    => html! { <p>{ "About" }</p> },
        SubRoute::NotFound => html! { <p>{ "Not Found" }</p> },
    }
}

// for {username}.github.io/{repo_name}, replace 'inyew' to your repo name
// XXX: remove all "/inyew" for {username}.github.io

#[derive(Clone, Routable, PartialEq)] enum MainRoute {
    #[at("/")] Home,
    #[at("/:s")] Subs,
}

#[derive(Clone, Routable, PartialEq)] enum SubRoute {
    #[at("/about")] About,
    #[at("/404")] #[not_found] NotFound,
}

#[function_component(Game24App)] fn game24app() -> Html {   // main root
    html! {
        <BrowserRouter basename="/inyew/">  // OR <base href="/inyew/"> in index.html?
            <Switch<MainRoute> render={ main_route }/>
        </BrowserRouter>
    }
}

fn main() {     // entry point
    wasm_logger::init(wasm_logger::Config::default()); //log::info!("Update: {:?}", msg);
    yew::Renderer::<Game24App>::new().render();
}
