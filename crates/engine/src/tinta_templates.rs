//! Tinta Office: the Tinta Presenta starter decks for File › New, with a frieze of azulejo tiles.
//!
//! Each deck uses one of four Tinta themes and decorates its master and layouts with tiles drawn
//! in code: a quatrefoil in the middle of each tile and quarter circles in the corners, which join
//! into full circles where four tiles meet. The motif, the colours and every word are our own
//! work (see `ATTRIBUTION.md`); nothing is copied from another product's templates or from a
//! photograph of real tiles.
//!
//! The slide text is English, Spanish or Ukrainian: `file.new` takes the UI language as `lang`.
//!
//! The tiles take their colours from the theme's scheme slots (accent 1 and 2, background 1), so
//! a later Design › Themes or Variants change recolours them. They are marked decorative, so
//! screen readers skip them.
//!
//! This file is fork-only: upstream's `templates.rs` calls it from three small hooks
//! ([`STARTERS`], [`theme`], [`decorate`]), so upstream syncs touch only those lines.

use std::fmt::Write as _;
use std::sync::Arc;

use deckcraft_color::{ColorScheme, Rgba, SchemeSlot};
use deckcraft_model::style::{ColorRef, Fill, Line};
use deckcraft_model::theme::{FontScheme, FontSet, Theme, default_format};
use deckcraft_model::{CustomPath, Geom, LayoutType, Presentation, Shape, ShapeId, Xfrm};

use crate::templates::{SlideSeed, Starter};

const fn seed(layout: &'static str, title: &'static str, bodies: &'static [&'static str], notes: &'static str) -> SlideSeed {
    SlideSeed { layout, title, bodies, notes }
}

/// The Tinta starters, shown before upstream's in the gallery.
pub const STARTERS: &[Starter] = &[
    Starter {
        name: "Azulejo Talk",
        summary: "A talk or presentation with one clear idea, framed in cobalt tiles",
        theme: "Tinta Cobalt",
        slides: &[
            seed("title", "A clear idea, well told", &["Your name · Place · Date"], "Say who you are and why this matters to the room."),
            seed(
                "titleAndContent",
                "What you will take away",
                &["One idea you can repeat tomorrow\n\tIn a sentence of your own\nOne example that shows it\nOne thing to try this week"],
                "",
            ),
            seed("sectionHeader", "The idea", &["Said simply, before the details"], ""),
            seed(
                "twoContent",
                "Before and after",
                &["Before\n\tWhat people do today\n\tWhat it costs them", "After\n\tWhat changes\n\tHow they will notice"],
                "",
            ),
            seed("title", "Thank you", &["Questions, and where to find us"], "Leave the contact slide up while you take questions."),
        ],
    },
    Starter {
        name: "Azulejo Report",
        summary: "A quarterly or yearly report on income, work and next steps, in warm ochre tiles",
        theme: "Tinta Ochre",
        slides: &[
            seed("title", "Quarterly report", &["Activity, income and next steps"], ""),
            seed(
                "titleAndContent",
                "At a glance",
                &["Income\n\tCompared with the same quarter last year\nNew clients\n\tAnd the clients who came back\nWork delivered on time"],
                "Three headlines. The details follow.",
            ),
            seed("sectionHeader", "The numbers", &["Only the ones that change a decision"], ""),
            seed(
                "comparison",
                "Plan and result",
                &["Planned", "What we set out to do\nThe budget we agreed", "Achieved", "What got done\nWhat it really cost"],
                "",
            ),
            seed(
                "titleAndContent",
                "Next quarter",
                &["Keep\n\tWhat worked and pays its way\nChange\n\tThe one thing that slowed us down\nStop\n\tWork that nobody asked for"],
                "",
            ),
            seed("title", "Questions", &["Thank you for reading"], ""),
        ],
    },
    Starter {
        name: "Azulejo Proposal",
        summary: "A proposal for a client: the need, the work, the timeline and the price, in green tiles",
        theme: "Tinta Olive",
        slides: &[
            seed("title", "Proposal for Client name", &["Prepared by Your business · Date"], ""),
            seed(
                "titleAndContent",
                "What you need",
                &["The problem in your words\n\tAs we heard it in our first conversation\nWhat a good result looks like\nWhen you need it"],
                "Check this slide with the client first: if the need is wrong, the rest is too.",
            ),
            seed(
                "twoContent",
                "Our proposal",
                &[
                    "We will\n\tStudy what you have today\n\tDo the work in clear stages\n\tShow you each stage",
                    "You receive\n\tThe finished work\n\tA short guide to use it\n\tA month of support",
                ],
                "",
            ),
            seed(
                "titleAndContent",
                "Timeline",
                &["Week 1\n\tStart and agree the details\nWeeks 2 to 4\n\tThe work, with a check-in every week\nWeek 5\n\tDelivery and handover"],
                "",
            ),
            seed(
                "titleAndContent",
                "Price and terms",
                &[
                    "A fixed price for the work described\n\tTaxes shown separately on the invoice\nPayment\n\tHalf at the start, half on delivery\nThis proposal is valid for 30 days",
                ],
                "",
            ),
            seed("title", "Next step", &["Reply to this proposal and we start on the date you choose"], ""),
        ],
    },
    Starter {
        name: "Azulejo Workshop",
        summary: "A hands-on class or workshop on a dark background with night-blue tiles",
        theme: "Tinta Night",
        slides: &[
            seed("title", "Workshop title", &["A hands-on session · two hours"], ""),
            seed(
                "titleAndContent",
                "Today",
                &["Why it matters\n\tTen minutes\nShow, then try\n\tAn hour, in pairs\nWhat you will do next\n\tTen minutes, at the end"],
                "",
            ),
            seed("sectionHeader", "Part one", &["Watch first, then do it yourself"], ""),
            seed(
                "titleAndContent",
                "Your turn",
                &["Work in pairs\n\tOne does, one watches and asks\nSwap after ten minutes\nWrite down the step that surprised you"],
                "Walk the room while they work.",
            ),
            seed("title", "Before you go", &["One thing you will try this week, written on a card"], ""),
        ],
    },
];

/// A Tinta starter's text in another language: the same layouts, slide for slide.
struct Local {
    lang: &'static str,
    name: &'static str,
    summary: &'static str,
    slides: &'static [SlideSeed],
}

/// Spanish and Ukrainian text of the Tinta starters, written from the meaning of the English
/// (AI-assisted, native-speaker review pending, as for the UI translations).
const LOCAL: &[Local] = &[
    Local {
        lang: "es",
        name: "Azulejo Talk",
        summary: "Una charla o presentación con una idea clara, enmarcada en azulejos cobalto",
        slides: &[
            seed("title", "Una idea clara, bien contada", &["Su nombre · Lugar · Fecha"], "Diga quién es y por qué esto importa a quien le escucha."),
            seed(
                "titleAndContent",
                "Lo que se llevará",
                &["Una idea que pueda repetir mañana\n\tCon sus propias palabras\nUn ejemplo que la muestre\nAlgo que probar esta semana"],
                "",
            ),
            seed("sectionHeader", "La idea", &["Dicha con sencillez, antes de los detalles"], ""),
            seed(
                "twoContent",
                "Antes y después",
                &["Antes\n\tLo que se hace hoy\n\tLo que cuesta", "Después\n\tLo que cambia\n\tCómo se notará"],
                "",
            ),
            seed("title", "Gracias", &["Preguntas, y dónde encontrarnos"], "Deje esta diapositiva a la vista mientras responde a las preguntas."),
        ],
    },
    Local {
        lang: "es",
        name: "Azulejo Report",
        summary: "Un informe trimestral o anual de ingresos, trabajo y próximos pasos, en azulejos ocre",
        slides: &[
            seed("title", "Informe trimestral", &["Actividad, ingresos y próximos pasos"], ""),
            seed(
                "titleAndContent",
                "De un vistazo",
                &[
                    "Ingresos\n\tComparados con el mismo trimestre del año pasado\nClientes nuevos\n\tY los que volvieron\nTrabajos entregados a tiempo",
                ],
                "Tres titulares. Los detalles vienen después.",
            ),
            seed("sectionHeader", "Las cifras", &["Solo las que cambian una decisión"], ""),
            seed(
                "comparison",
                "Plan y resultado",
                &["Previsto", "Lo que nos propusimos\nEl presupuesto acordado", "Conseguido", "Lo que se hizo\nLo que costó de verdad"],
                "",
            ),
            seed(
                "titleAndContent",
                "Próximo trimestre",
                &["Mantener\n\tLo que funcionó y se paga solo\nCambiar\n\tLo único que nos frenó\nDejar\n\tEl trabajo que nadie pidió"],
                "",
            ),
            seed("title", "Preguntas", &["Gracias por su atención"], ""),
        ],
    },
    Local {
        lang: "es",
        name: "Azulejo Proposal",
        summary: "Una propuesta para un cliente: la necesidad, el trabajo, los plazos y el precio, en azulejos verdes",
        slides: &[
            seed("title", "Propuesta para Nombre del cliente", &["Preparada por Su negocio · Fecha"], ""),
            seed(
                "titleAndContent",
                "Lo que necesita",
                &[
                    "El problema, en sus palabras\n\tTal como lo entendimos en la primera conversación\nCómo sería un buen resultado\nPara cuándo lo necesita",
                ],
                "Revise primero esta diapositiva con el cliente: si la necesidad no es la correcta, el resto tampoco.",
            ),
            seed(
                "twoContent",
                "Nuestra propuesta",
                &[
                    "Nosotros\n\tEstudiaremos lo que tiene hoy\n\tHaremos el trabajo por etapas claras\n\tLe mostraremos cada etapa",
                    "Usted recibe\n\tEl trabajo terminado\n\tUna guía breve para usarlo\n\tUn mes de soporte",
                ],
                "",
            ),
            seed(
                "titleAndContent",
                "Calendario",
                &[
                    "Semana 1\n\tInicio y acuerdo de los detalles\nSemanas 2 a 4\n\tEl trabajo, con una revisión cada semana\nSemana 5\n\tEntrega y traspaso",
                ],
                "",
            ),
            seed(
                "titleAndContent",
                "Precio y condiciones",
                &[
                    "Un precio cerrado por el trabajo descrito\n\tEl IVA aparece aparte en la factura\nPago\n\tLa mitad al empezar, la mitad a la entrega\nEsta propuesta es válida durante 30 días",
                ],
                "",
            ),
            seed("title", "Siguiente paso", &["Responda a esta propuesta y empezamos en la fecha que usted elija"], ""),
        ],
    },
    Local {
        lang: "es",
        name: "Azulejo Workshop",
        summary: "Una clase o taller práctico sobre fondo oscuro con azulejos azul noche",
        slides: &[
            seed("title", "Título del taller", &["Una sesión práctica · dos horas"], ""),
            seed(
                "titleAndContent",
                "Hoy",
                &[
                    "Por qué importa\n\tDiez minutos\nPrimero lo vemos, luego lo prueba\n\tUna hora, por parejas\nQué hará después\n\tDiez minutos, al final",
                ],
                "",
            ),
            seed("sectionHeader", "Primera parte", &["Primero mire, luego hágalo usted"], ""),
            seed(
                "titleAndContent",
                "Su turno",
                &["Trabajen por parejas\n\tUno hace, el otro observa y pregunta\nCambien a los diez minutos\nApunte el paso que le sorprendió"],
                "Pase por las mesas mientras trabajan.",
            ),
            seed("title", "Antes de irse", &["Una cosa que probará esta semana, escrita en una tarjeta"], ""),
        ],
    },
    Local {
        lang: "uk",
        name: "Azulejo Talk",
        summary: "Виступ або презентація з однією ясною ідеєю в рамці з кобальтових плиток",
        slides: &[
            seed("title", "Ясна ідея, добре розказана", &["Ваше ім'я · Місце · Дата"], "Скажіть, хто ви і чому це важливо для слухачів."),
            seed(
                "titleAndContent",
                "Що ви винесете",
                &[
                    "Одну ідею, яку завтра зможете повторити\n\tСвоїми словами\nОдин приклад, що її показує\nОдну річ, яку варто спробувати цього тижня",
                ],
                "",
            ),
            seed("sectionHeader", "Ідея", &["Просто, перед подробицями"], ""),
            seed(
                "twoContent",
                "До і після",
                &["До\n\tЩо люди роблять сьогодні\n\tЧого це їм коштує", "Після\n\tЩо зміниться\n\tЯк вони це помітять"],
                "",
            ),
            seed("title", "Дякую", &["Запитання й де нас знайти"], "Залиште цей слайд на екрані, поки відповідаєте на запитання."),
        ],
    },
    Local {
        lang: "uk",
        name: "Azulejo Report",
        summary: "Квартальний чи річний звіт про доходи, роботу й наступні кроки, у вохристих плитках",
        slides: &[
            seed("title", "Квартальний звіт", &["Діяльність, доходи й наступні кроки"], ""),
            seed(
                "titleAndContent",
                "Коротко",
                &["Доходи\n\tПорівняно з тим самим кварталом торік\nНові клієнти\n\tІ ті, хто повернувся\nРоботи, здані вчасно"],
                "Три головні речі. Подробиці далі.",
            ),
            seed("sectionHeader", "Цифри", &["Лише ті, що змінюють рішення"], ""),
            seed(
                "comparison",
                "План і результат",
                &["Заплановано", "Що ми збиралися зробити\nБюджет, про який домовилися", "Досягнуто", "Що зроблено\nСкільки це коштувало насправді"],
                "",
            ),
            seed(
                "titleAndContent",
                "Наступний квартал",
                &[
                    "Залишаємо\n\tТе, що спрацювало й окупається\nЗмінюємо\n\tТе одне, що нас гальмувало\nПрипиняємо\n\tРоботу, про яку ніхто не просив",
                ],
                "",
            ),
            seed("title", "Запитання", &["Дякуємо за увагу"], ""),
        ],
    },
    Local {
        lang: "uk",
        name: "Azulejo Proposal",
        summary: "Пропозиція клієнтові: потреба, робота, терміни й ціна, у зелених плитках",
        slides: &[
            seed("title", "Пропозиція для «Назва клієнта»", &["Підготувала Ваша компанія · Дата"], ""),
            seed(
                "titleAndContent",
                "Що вам потрібно",
                &["Проблема вашими словами\n\tЯк ми її почули під час першої розмови\nЯким має бути добрий результат\nКоли вам це потрібно"],
                "Спершу перевірте цей слайд із клієнтом: якщо потребу описано неправильно, решта теж буде хибною.",
            ),
            seed(
                "twoContent",
                "Наша пропозиція",
                &[
                    "Ми\n\tВивчимо, що у вас є зараз\n\tЗробимо роботу чіткими етапами\n\tПокажемо вам кожен етап",
                    "Ви отримаєте\n\tГотову роботу\n\tКоротку інструкцію до неї\n\tМісяць підтримки",
                ],
                "",
            ),
            seed(
                "titleAndContent",
                "Графік",
                &["Тиждень 1\n\tСтарт і узгодження деталей\nТижні 2–4\n\tРобота з перевіркою щотижня\nТиждень 5\n\tЗдача й передання"],
                "",
            ),
            seed(
                "titleAndContent",
                "Ціна й умови",
                &[
                    "Фіксована ціна за описану роботу\n\tПодатки окремим рядком у рахунку\nОплата\n\tПоловина на початку, половина після здачі\nПропозиція дійсна 30 днів",
                ],
                "",
            ),
            seed("title", "Наступний крок", &["Дайте відповідь на цю пропозицію, і ми почнемо в обраний вами день"], ""),
        ],
    },
    Local {
        lang: "uk",
        name: "Azulejo Workshop",
        summary: "Практичне заняття чи майстер-клас на темному тлі з нічно-синіми плитками",
        slides: &[
            seed("title", "Назва майстер-класу", &["Практичне заняття · дві години"], ""),
            seed(
                "titleAndContent",
                "Сьогодні",
                &["Чому це важливо\n\tДесять хвилин\nПоказуємо, потім пробуєте\n\tГодина, у парах\nЩо робитимете далі\n\tДесять хвилин наприкінці"],
                "",
            ),
            seed("sectionHeader", "Частина перша", &["Спершу дивіться, потім робіть самі"], ""),
            seed(
                "titleAndContent",
                "Ваша черга",
                &["Працюйте в парах\n\tОдин робить, другий дивиться й питає\nЗа десять хвилин поміняйтеся\nЗапишіть крок, який вас здивував"],
                "Ходіть між учасниками, поки вони працюють.",
            ),
            seed("title", "Перед тим як піти", &["Одна річ, яку спробуєте цього тижня, записана на картці"], ""),
        ],
    },
];

/// The text of `starter` in `lang` (`es`, `uk`, or a tag starting with one, such as `es-ES`).
fn local(starter: &str, lang: &str) -> Option<&'static Local> {
    let lang = lang.trim().to_ascii_lowercase();
    LOCAL.iter().find(|l| l.name == starter && (lang == l.lang || lang.strip_prefix(l.lang).is_some_and(|rest| rest.starts_with(['-', '_']))))
}

/// The slides of the Tinta starter `starter` in `lang`, when it has that language.
pub fn slides(starter: &str, lang: &str) -> Option<&'static [SlideSeed]> {
    local(starter, lang).map(|l| l.slides)
}

/// The gallery summary of the Tinta starter `starter` in `lang`, when it has that language.
pub fn summary(starter: &str, lang: &str) -> Option<&'static str> {
    local(starter, lang).map(|l| l.summary)
}

fn hex(s: &str) -> Rgba {
    Rgba::from_hex(s).unwrap_or(Rgba::BLACK)
}

/// A Tinta theme: `colors` are dk1, lt1, dk2, lt2, accents 1–6, hyperlink, followed hyperlink.
fn tinta_theme(name: &str, colors: [&str; 12], major: &str, minor: &str) -> Theme {
    Theme {
        name: name.into(),
        colors: ColorScheme::new(name, colors.map(hex)),
        fonts: FontScheme {
            name: name.into(),
            major: FontSet { latin: major.into(), ..Default::default() },
            minor: FontSet { latin: minor.into(), ..Default::default() },
        },
        format: default_format(),
        raw_extra: None,
    }
}

/// The themes the Tinta starters use. Their fonts all cover Latin and Cyrillic.
pub fn themes() -> Vec<Theme> {
    vec![
        tinta_theme(
            "Tinta Cobalt",
            ["1B2433", "FFFFFF", "1D3A6E", "EEF2F8", "1F4E9A", "D99A1E", "2E7D6B", "B5482E", "5B7DB8", "8C6A3F", "1F4E9A", "6B4E9A"],
            "Merriweather",
            "Open Sans",
        ),
        tinta_theme(
            "Tinta Ochre",
            ["2A2118", "FFFDF8", "5A3A1E", "F6EEDF", "A9432B", "D39A2F", "1F4E9A", "2E7D6B", "7A5C3E", "C9785B", "A9432B", "7A5C3E"],
            "Playfair Display",
            "Lato",
        ),
        tinta_theme(
            "Tinta Olive",
            ["1C241C", "FFFFFF", "2F4A3A", "EEF3EC", "2E6B4F", "D39A2F", "1F4E9A", "A9432B", "6E8F5A", "4F6D7A", "2E6B4F", "4F6D7A"],
            "Montserrat",
            "Lato",
        ),
        // A dark theme: lt1 (the background) is the night blue and dk1 (the text) is light.
        tinta_theme(
            "Tinta Night",
            ["F5F1E8", "14213D", "F2E6C9", "1E2F52", "6E9BE0", "F0B93C", "4FB39A", "E07A5F", "A3B8DC", "C9A86A", "8AB4F8", "C3A6E8"],
            "Poppins",
            "Nunito Sans",
        ),
    ]
}

/// The Tinta theme called `name`, for a Tinta starter.
pub fn theme(name: &str) -> Option<Theme> {
    themes().into_iter().find(|t| t.name.eq_ignore_ascii_case(name))
}

/// Whether `name` is one of the Tinta starters.
fn is_tinta(name: &str) -> bool {
    STARTERS.iter().any(|t| t.name == name)
}

/// The cubic control-point factor for a quarter circle.
const K: f64 = 0.552_284_75;

/// A quarter disc of radius `r` whose centre is the corner (`cx`, `cy`); `dx`, `dy` (±1) point
/// into the tile.
fn quarter(d: &mut String, cx: f64, cy: f64, dx: f64, dy: f64, r: f64) {
    let (ax, ay) = (cx + dx * r, cy);
    let (bx, by) = (cx, cy + dy * r);
    let _ = write!(d, "M {cx:.2} {cy:.2} L {ax:.2} {ay:.2} C {:.2} {:.2} {:.2} {:.2} {bx:.2} {by:.2} Z ", ax, cy + dy * r * K, cx + dx * r * K, by,);
}

/// A full disc of radius `r` centred on (`cx`, `cy`).
fn disc(d: &mut String, cx: f64, cy: f64, r: f64) {
    let k = r * K;
    let _ = write!(
        d,
        "M {:.2} {cy:.2} C {:.2} {:.2} {:.2} {:.2} {cx:.2} {:.2} C {:.2} {:.2} {:.2} {:.2} {:.2} {cy:.2} \
         C {:.2} {:.2} {:.2} {:.2} {cx:.2} {:.2} C {:.2} {:.2} {:.2} {:.2} {:.2} {cy:.2} Z ",
        cx + r,
        cx + r,
        cy + k,
        cx + k,
        cy + r,
        cy + r,
        cx - k,
        cy + r,
        cx - r,
        cy + k,
        cx - r,
        cx - r,
        cy - k,
        cx - k,
        cy - r,
        cy - r,
        cx + k,
        cy - r,
        cx + r,
        cy - k,
        cx + r,
    );
}

/// A four-pointed star with curved sides, its points at the middle of each side of the tile.
fn quatrefoil(d: &mut String, x: f64, y: f64, s: f64) {
    let (cx, cy) = (x + s / 2.0, y + s / 2.0);
    let reach = s * 0.42;
    let pinch = s * 0.12;
    let _ = write!(
        d,
        "M {cx:.2} {:.2} Q {:.2} {:.2} {:.2} {cy:.2} Q {:.2} {:.2} {cx:.2} {:.2} Q {:.2} {:.2} {:.2} {cy:.2} Q {:.2} {:.2} {cx:.2} {:.2} Z ",
        cy - reach,
        cx + pinch,
        cy - pinch,
        cx + reach,
        cx + pinch,
        cy + pinch,
        cy + reach,
        cx - pinch,
        cy + pinch,
        cx - reach,
        cx - pinch,
        cy - pinch,
        cy - reach,
    );
}

/// The three colour layers of a run of `cols` × `rows` tiles of side `s`, in the band's own
/// coordinates: the deep motif (corner quarters and the star), the bright centres, and the
/// thin grid between tiles (drawn under the motif).
fn tile_paths(cols: u32, rows: u32, s: f64) -> [String; 3] {
    let (mut deep, mut bright, mut grid) = (String::new(), String::new(), String::new());
    for row in 0..rows {
        for col in 0..cols {
            let (x, y) = (f64::from(col) * s, f64::from(row) * s);
            for (cx, cy, dx, dy) in [(x, y, 1.0, 1.0), (x + s, y, -1.0, 1.0), (x, y + s, 1.0, -1.0), (x + s, y + s, -1.0, -1.0)] {
                quarter(&mut deep, cx, cy, dx, dy, s * 0.28);
                quarter(&mut bright, cx, cy, dx, dy, s * 0.12);
            }
            quatrefoil(&mut deep, x, y, s);
            disc(&mut bright, x + s / 2.0, y + s / 2.0, s * 0.09);
        }
    }
    let (w, h) = (f64::from(cols) * s, f64::from(rows) * s);
    for col in 0..=cols {
        let x = f64::from(col) * s;
        let _ = write!(grid, "M {x:.2} 0 L {x:.2} {h:.2} ");
    }
    for row in 0..=rows {
        let y = f64::from(row) * s;
        let _ = write!(grid, "M 0 {y:.2} L {w:.2} {y:.2} ");
    }
    [deep, bright, grid]
}

/// The shapes of one band of tiles at (`x`, `y`) on a 960 × 540 slide, scaled by `sx`, `sy`.
/// `None` when the deck has no free ids.
fn band(doc: &mut Presentation, name: &str, at: (f64, f64), cols: u32, rows: u32, s: f64, scale: (f64, f64)) -> Option<Vec<Shape>> {
    let (w, h) = (f64::from(cols) * s, f64::from(rows) * s);
    let xfrm = Xfrm::new(at.0 * scale.0, at.1 * scale.1, w * scale.0, h * scale.1);
    let [deep, bright, grid] = tile_paths(cols, rows, s);
    let path = |d: String, stroke: bool| Geom::Custom { paths: vec![CustomPath { w, h, d, stroke, ..Default::default() }] };
    let color = |slot| ColorRef::scheme(slot);
    let layers = [
        ("ground", Geom::default(), Some(Fill::solid(color(SchemeSlot::Bg1))), Line::none()),
        (
            "grid",
            path(grid, true),
            Some(Fill::None),
            Line::solid(color(SchemeSlot::Accent1).with(deckcraft_color::ColorTransform::Alpha(45000)), 0.75),
        ),
        ("motif", path(deep, false), Some(Fill::solid(color(SchemeSlot::Accent1))), Line::none()),
        ("centres", path(bright, false), Some(Fill::solid(color(SchemeSlot::Accent2))), Line::none()),
    ];
    let mut out = vec![];
    for (part, geom, fill, line) in layers {
        out.push(Shape {
            id: ShapeId(doc.alloc_id()?),
            name: format!("{name} {part}"),
            xfrm: Some(xfrm),
            geom,
            fill,
            line: Some(line),
            descr: String::new(),
            decorative: true,
            ..Default::default()
        });
    }
    Some(out)
}

/// Add the azulejo tiles to a deck built from a Tinta starter; other decks are left alone.
///
/// Every layout shows a narrow column of tiles down the left edge (on the master), outside the
/// placeholders. The title slide and section header hide it and show wider bands instead: the
/// title slide one along the top and one along the bottom, the section header one along the top.
pub fn decorate(starter: &str, doc: &mut Presentation) {
    if !is_tinta(starter) {
        return;
    }
    let size = doc.slide_size;
    let scale = (size.width / 960.0, size.height / 540.0);
    let Some(master) = doc.masters.first().cloned() else { return };
    let mut master = (*master).clone();

    // The master: a column of 27-point tiles, 20 of them filling the 540-point height.
    let Some(column) = band(doc, "Azulejo column", (0.0, 0.0), 1, 20, 27.0, scale) else { return };
    // Behind the placeholders.
    master.shapes.splice(0..0, column);

    for i in 0..master.layouts.len() {
        let bands: &[(&str, (f64, f64))] = match master.layouts[i].kind {
            LayoutType::Title => &[("Azulejo top", (0.0, 0.0)), ("Azulejo bottom", (0.0, 480.0))],
            LayoutType::SectionHeader => &[("Azulejo top", (0.0, 0.0))],
            _ => continue,
        };
        let mut shapes = vec![];
        for (name, at) in bands {
            let Some(b) = band(doc, name, *at, 16, 1, 60.0, scale) else { return };
            shapes.extend(b);
        }
        let layout = &mut master.layouts[i];
        layout.show_master_shapes = false;
        layout.shapes.splice(0..0, shapes);
    }
    doc.masters[0] = Arc::new(master);
    // The bottom band is where the footer row sits on the title slide.
    doc.header_footer.hide_on_title = true;
}

#[cfg(test)]
mod tests {
    use deckcraft_model::defaults;

    use super::*;
    use crate::templates;

    fn decor(shapes: &[Shape]) -> Vec<&Shape> {
        shapes.iter().filter(|sh| sh.decorative && sh.name.starts_with("Azulejo")).collect()
    }

    #[test]
    fn tinta_starters_come_first_and_each_has_its_own_theme() {
        let names: Vec<&str> = templates::starters().iter().map(|t| t.name).collect();
        assert_eq!(&names[..STARTERS.len()], STARTERS.iter().map(|t| t.name).collect::<Vec<_>>().as_slice());
        assert!(names.len() > STARTERS.len(), "upstream's starters are still listed");
        let mut themes: Vec<&str> = STARTERS.iter().map(|t| t.theme).collect();
        themes.dedup();
        assert_eq!(themes.len(), STARTERS.len());
        assert_eq!(templates::matching("azulejo").len(), STARTERS.len());
    }

    #[test]
    fn tiles_stay_outside_every_placeholder_at_any_slide_size() {
        for size in [defaults::WIDE, defaults::STANDARD] {
            for starter in STARTERS {
                let doc = templates::build(starter.name, size).unwrap();
                let master = &doc.masters[0];
                let eps = 1e-6;
                let tiles_of = |shapes: &[Shape]| decor(shapes).iter().filter_map(|sh| sh.xfrm.as_ref().map(Xfrm::rect)).collect::<Vec<_>>();
                assert!(!tiles_of(&master.shapes).is_empty(), "{} master has tiles", starter.name);
                for layout in &master.layouts {
                    let mut tiles = tiles_of(&layout.shapes);
                    if layout.show_master_shapes {
                        tiles.extend(tiles_of(&master.shapes));
                    } else {
                        assert!(!tiles.is_empty(), "{} {} hides the column and has bands", starter.name, layout.name);
                    }
                    let boxes = layout
                        .shapes
                        .iter()
                        .filter(|sh| sh.ph_type().is_some_and(|k| !k.is_footer_kind()))
                        .filter_map(|sh| sh.xfrm.as_ref().map(Xfrm::rect));
                    for ph in boxes {
                        for tile in &tiles {
                            assert!(tile.intersect(ph).area() < eps, "{} {}: tiles {tile:?} overlap a placeholder {ph:?}", starter.name, layout.name);
                        }
                    }
                    for tile in &tiles {
                        let inside = tile.x0 > -eps && tile.y0 > -eps && tile.x1 < size.width + eps && tile.y1 < size.height + eps;
                        assert!(inside, "{} {}: tiles {tile:?} leave the slide", starter.name, layout.name);
                    }
                }
            }
        }
    }

    #[test]
    fn tiles_follow_the_theme_and_survive_pptx() {
        let doc = templates::build("Azulejo Proposal", defaults::WIDE).unwrap();
        let motif = doc.masters[0].shapes.iter().find(|sh| sh.name.ends_with("motif")).unwrap();
        assert_eq!(motif.fill, Some(Fill::solid(ColorRef::scheme(SchemeSlot::Accent1))));
        let bytes = deckcraft_pptx::export(&doc).unwrap();
        let back = deckcraft_pptx::import(&bytes).unwrap();
        let count = |p: &Presentation| decor(&p.masters[0].shapes).len() + p.masters[0].layouts.iter().map(|l| decor(&l.shapes).len()).sum::<usize>();
        assert!(count(&doc) > 0);
        assert_eq!(count(&back), count(&doc), "every tile layer comes back from .pptx");
        let title = back.masters[0].layouts.iter().find(|l| l.kind == LayoutType::Title).unwrap();
        assert!(!title.show_master_shapes);
        assert!(title.shapes.iter().any(|sh| matches!(sh.geom, Geom::Custom { .. })));
    }

    #[test]
    fn upstream_starters_get_no_tiles() {
        let doc = templates::build("Briefing", defaults::WIDE).unwrap();
        assert!(decor(&doc.masters[0].shapes).is_empty());
        assert!(doc.masters[0].layouts.iter().all(|l| decor(&l.shapes).is_empty() && l.show_master_shapes));
    }

    #[test]
    fn spanish_and_ukrainian_text_fills_the_same_slides() {
        for lang in ["es", "uk", "es-ES"] {
            for starter in STARTERS {
                let local = slides(starter.name, lang).unwrap_or_else(|| panic!("{} has no {lang} text", starter.name));
                assert_eq!(local.len(), starter.slides.len(), "{} {lang}", starter.name);
                for (en, other) in starter.slides.iter().zip(local) {
                    assert_eq!(other.layout, en.layout, "{} {lang}", starter.name);
                    assert_eq!(other.bodies.len(), en.bodies.len(), "{} {lang}: {}", starter.name, other.title);
                    let levels = |t: &str| t.lines().map(|l| l.chars().take_while(|c| *c == '\t').count()).collect::<Vec<_>>();
                    for (a, b) in en.bodies.iter().zip(other.bodies) {
                        assert_eq!(levels(b), levels(a), "{} {lang}: outline of {b:?}", starter.name);
                    }
                    assert_eq!(other.notes.is_empty(), en.notes.is_empty(), "{} {lang}: notes of {}", starter.name, other.title);
                }
                assert!(summary(starter.name, lang).is_some());
                let doc = templates::build_in(starter.name, defaults::WIDE, lang).unwrap();
                assert_eq!(doc.slides[0].title(), local[0].title);
                assert_ne!(doc.slides[0].title(), starter.slides[0].title, "{} {lang} is translated", starter.name);
            }
        }
    }

    #[test]
    fn other_languages_and_upstream_starters_stay_english() {
        for lang in ["", "en", "fr", "esperanto", "ukr"] {
            let doc = templates::build_in("Azulejo Report", defaults::WIDE, lang).unwrap();
            assert_eq!(doc.slides[0].title(), STARTERS[1].slides[0].title, "{lang:?}");
        }
        let briefing = templates::build_in("Briefing", defaults::WIDE, "es").unwrap();
        assert_eq!(briefing.slides[0].title(), "Weekly briefing");
        let mut s = crate::Session::new();
        s.execute("file.new", &serde_json::json!({"template": "Azulejo Talk", "lang": "uk"})).unwrap();
        assert_eq!(s.active().unwrap().doc.slides[0].title(), "Ясна ідея, добре розказана");
    }
}
