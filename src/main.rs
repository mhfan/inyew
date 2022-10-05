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

#[function_component(Game24)] fn game24() -> Html {
    let court = [ "T", "J", "Q", "K" ];
    let suits = [ "S", "C", "D", "H" ];     // "♣♦♥♠"
    let mut poker= (0..52).collect::<Vec<_>>();

    let (goal, cnt) = (24, 4);
    use rand::{thread_rng, seq::SliceRandom};
    let mut rng = thread_rng();

    poker.shuffle(&mut rng);    let pkns;
    let mut _rems = poker.as_mut_slice();
    (pkns, _rems) = _rems.partial_shuffle(&mut rng, cnt);

    use yew::TargetCast;
    use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlButtonElement};

    let ops_selected = Callback::from(|e: Event| {
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
        e.target_dyn_into::<HtmlInputElement>().map(|inp| {
            log::info!("{}", inp.get_attribute("id").unwrap());
            inp.value()   // add value='xxx' in <input> element to change default('on')
        });
    });

    let cnt_changed = Callback::from(|e: Event| {
        e.target_dyn_into::<HtmlSelectElement>().map(|sel| {
            // TODO: cnt = sel.value(); -> then 'pkns' then 'nums'
            log::info!("{}", sel.value());  sel.value()
        });
    });

    let num_selected = Callback::from(|e: FocusEvent| {
        e.target_dyn_into::<HtmlButtonElement>().map(|btn| {
            log::info!("{}", btn.inner_text());     btn.inner_text()
        });
    });

    let num_class = "px-4 py-2 m-4 w-16 text-2xl text-purple-600 font-semibold border border-purple-200 hover:text-white hover:bg-purple-600 hover:border-transparent focus:outline-none focus:ring-2 focus:ring-purple-600 focus:ring-offset-2 shadow-xl";
    let nums = pkns.iter().map(|pkn| {
        let (num, sid) = ((pkn % 13) + 1, (pkn / 13)/* % 4 */);
        let _ = format!(r"{}{}.svg", match num { 1 => "A".to_owned(),
            2..=9 => num.to_string(), 10..=13 => court[num - 10].to_owned(),
            _ => "?".to_owned() }, suits[sid]);     //num

        html!{
            <button class={classes!(num_class, "rounded-full")} data-bs-toggle="tooltip" title="Click to select/unselect\nDrag over to exchange\nDouble click to input new number">{ num.to_string() }</button>
            // https://en.wikipedia.org/wiki/Playing_cards_in_Unicode
        }
    }).collect::<Html>();

    let ops = [ "+", "-", "×", "÷" ].into_iter().map(|op| html!{
        <div class="m-4">
            <input type="radio" id={ op } name="select-ops" class="hidden peer"/>
            <label for={ op } class="bg-indigo-600 hover:bg-indigo-500 text-white font-bold py-2 px-4 text-3xl peer-checked:outline-none peer-checked:ring-2 peer-checked:ring-indigo-500 peer-checked:ring-offset-2 peer-checked:bg-transparent rounded-md shadow-xl" data-bs-toggle="tooltip" title="Click to select/unselect\nDrag over to replace">{ op }</label>
        </div>
    }).collect::<Html>();

    let ctrl_class = "text-gray-900 bg-gradient-to-r from-lime-200 via-lime-400 to-lime-500 hover:bg-gradient-to-br focus:ring-4 focus:outline-none focus:ring-lime-300 dark:focus:ring-lime-800 shadow-lg shadow-lime-500/50 dark:shadow-lg dark:shadow-lime-800/80 font-bold rounded-lg px-4 py-2 m-4";

    let cnt_options = (4..=6).map(|n| html!{
        <option value={ n.to_string() } selected={ n == 4 }>{ format!("{n} nums") }</option>
    }).collect::<Html>();

    // https://stackoverflow.com/questions/62554142/how-to-select-multiple-button-options-on-same-html-page-using-tailwind-css
    html!{ <main>
        //<div id="play-cards"/>

        <div id="ops-group" onchange={ ops_selected } class="flex place-content-center">{ ops }</div>

        <div id="expr-skel" onfocus={ num_selected }>{ nums }
            <label class="text-white font-bold py-2 px-4 m-4 text-3xl rounded-md" data-bs-toggle="tooltip" title="Click to calculate">{ "≠?" }</label>
            <button class={classes!(num_class, "rounded-md")} data-bs-toggle="tooltip" title="Double click to input new goal">{ goal.to_string() }</button>
        </div>

        <div id="ctrl-btns">
            <select class={classes!(ctrl_class)} onchange={ cnt_changed } data-bs-toogle="tooltip" title="Click to select numbers count">{ cnt_options }</select>
            <button class={classes!(ctrl_class)} data-bs-toogle="tooltip" title="Click to break down selected compound expression">{ "Restore" }</button>
            <button class={classes!(ctrl_class)} data-bs-toogle="tooltip" title="Click to refresh new round game">{ "Refresh" }</button>
        </div>
    </main> }
}

fn root_route(routes: &RootRoute) -> Html {
    #[allow(clippy::let_unit_value)]
    match routes {
        RootRoute::Home  => html!{ <>
            //margin: 0 auto;   //class: justify-center;
            <style> { r" body { text-align: center; } " } </style>

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
