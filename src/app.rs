use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use time::OffsetDateTime;
use uuid::Uuid;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/soulcrush.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let solicitaties = RwSignal::new(mock_data());

    view! {
        <h1>"Job Applications"</h1>
        <SolicitatieTable solicitaties />
    }
}

fn mock_data() -> Vec<Solicitatie> {
    vec![
        Solicitatie {
            id: Uuid::new_v4(),
            company: Company::new(
                "Acme Corp".into(),
                "https://acme.com".into(),
                "John Doe".into(),
                "Technology".into(),
            ),
            status: Status::ToDo,
            date: OffsetDateTime::now_utc(),
        },
        Solicitatie {
            id: Uuid::new_v4(),
            company: Company::new(
                "TechStart".into(),
                "https://techstart.io".into(),
                "Jane Smith".into(),
                "Startup".into(),
            ),
            status: Status::Solicitated,
            date: OffsetDateTime::now_utc(),
        },
        Solicitatie {
            id: Uuid::new_v4(),
            company: Company::new(
                "BigBank".into(),
                "https://bigbank.com".into(),
                "Bob Wilson".into(),
                "Finance".into(),
            ),
            status: Status::Pending,
            date: OffsetDateTime::now_utc(),
        },
    ]
}

#[component]
fn SolicitatieTable(solicitaties: RwSignal<Vec<Solicitatie>>) -> impl IntoView {
    view! {
        <table class="solicitatie-table">
            <thead>
                <tr>
                    <th>"Company"</th>
                    <th>"Industry"</th>
                    <th>"Website"</th>
                    <th>"Status"</th>
                    <th>"Date"</th>
                    <th>"Actions"</th>
                </tr>
            </thead>
            <tbody>
                <For
                    each=move || solicitaties.get()
                    key=|s| s.id
                    let:solicitatie
                >
                    <SolicitatieRow solicitatie solicitaties />
                </For>
            </tbody>
        </table>
    }
}

#[component]
fn SolicitatieRow(
    solicitatie: Solicitatie,
    solicitaties: RwSignal<Vec<Solicitatie>>,
) -> impl IntoView {
    let id = solicitatie.id;
    let status = RwSignal::new(solicitatie.status);

    let on_status_click = move |_| {
        let new_status = status.get().next();
        status.set(new_status);
        solicitaties.update(|list| {
            if let Some(s) = list.iter_mut().find(|s| s.id == id) {
                s.status = new_status;
            }
        });
    };

    let on_delete = move |_| {
        solicitaties.update(|list| {
            list.retain(|s| s.id != id);
        });
    };

    let date_str = solicitatie.date.date().to_string();

    view! {
        <tr>
            <td>{solicitatie.company.name.clone()}</td>
            <td>{solicitatie.company.industry.clone()}</td>
            <td>
                <a href={solicitatie.company.website.clone()} target="_blank">
                    "Visit"
                </a>
            </td>
            <td>
                <button
                    class=move || format!("status-badge {}", status.get().css_class())
                    on:click=on_status_click
                >
                    {move || status.get().to_string()}
                </button>
            </td>
            <td>{date_str}</td>
            <td>
                <button class="btn-delete" on:click=on_delete>"Delete"</button>
            </td>
        </tr>
    }
}

#[derive(Clone, PartialEq)]
struct Solicitatie {
    id: Uuid,
    company: Company,
    status: Status,
    date: OffsetDateTime,
}

#[derive(Clone, PartialEq)]
struct Company {
    id: Uuid,
    name: String,
    website: String,
    ceo: String,
    industry: String,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum Status {
    #[default]
    ToDo,
    Solicitated,
    Pending,
    Accepted,
    Rejected,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::ToDo => write!(f, "To Do"),
            Status::Solicitated => write!(f, "Solicitated"),
            Status::Pending => write!(f, "Pending"),
            Status::Accepted => write!(f, "Accepted"),
            Status::Rejected => write!(f, "Rejected"),
        }
    }
}

impl Status {
    fn next(self) -> Self {
        match self {
            Status::ToDo => Status::Solicitated,
            Status::Solicitated => Status::Pending,
            Status::Pending => Status::Accepted,
            Status::Accepted => Status::Rejected,
            Status::Rejected => Status::ToDo,
        }
    }

    fn css_class(&self) -> &'static str {
        match self {
            Status::ToDo => "status-todo",
            Status::Solicitated => "status-solicitated",
            Status::Pending => "status-pending",
            Status::Accepted => "status-accepted",
            Status::Rejected => "status-rejected",
        }
    }
}

impl Solicitatie {
    pub fn new(company: Company) -> Self {
        let id = Uuid::new_v4();
        let status = Status::default();
        let date = OffsetDateTime::now_utc();

        Self {
            id,
            company,
            status,
            date,
        }
    }
}

impl Company {
    pub fn new(name: String, website: String, ceo: String, industry: String) -> Self {
        let id = Uuid::new_v4();

        Self {
            id,
            name,
            website,
            ceo,
            industry,
        }
    }
}
