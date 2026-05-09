use crate::components::atoms::accordion::{
    Accordion, AccordionContent, AccordionItem, AccordionTrigger,
};
use crate::components::atoms::card::{
    Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle,
};
use crate::components::atoms::{Button, Tooltip, TooltipContent, TooltipTrigger};
use crate::layouts::DefaultLayout;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::HiArrowRight;
use dioxus_free_icons::Icon;
use dioxus_i18n::{prelude::*, tid};
use dioxus_primitives::ContentSide;
use unic_langid::langid;

#[component]
pub fn Database() -> Element {
    use_init_i18n(|| {
        I18nConfig::new(langid!("en-US"))
            .with_locale((langid!("en-US"), include_str!("./locales/en-US.ftl")))
            .with_locale((langid!("de-DE"), include_str!("./locales/de-DE.ftl")))
    });

    rsx! {
        DefaultLayout {
            Accordion { class: "developer-actions",
                AccordionItem { index: 1, default_open: true,
                    AccordionTrigger { {tid!("developer-actions-database")} }
                    AccordionContent {
                        div { class: "grid grid-cols-1 md:grid-cols-2 md:gap-6 lg:grid-cols-3",

                            // Admin DB migrations
                            Card {
                                CardHeader {
                                    CardTitle { {tid!("developer-actions-database-migrations")} }
                                    CardDescription { {tid!("developer-actions-database-migrations.description")} }
                                }
                                CardContent {
                                    Tooltip {
                                        TooltipTrigger {
                                            CardAction {
                                                Button {
                                                    onclick: move |_| async move {
                                                        let _ = api::developer::migrate_database().await;
                                                    },
                                                    div { class: "flex flex-col items-center",
                                                        Icon {
                                                            fill: "black",
                                                            icon: HiArrowRight,
                                                            style: "color: var(--color-btn-icon)",
                                                        }
                                                        {tid!("developer-actions-database-migrations-run")}
                                                    }
                                                }
                                            }
                                        }
                                        TooltipContent { side: ContentSide::Bottom,
                                            p { {tid!("developer-actions-database-migrations-run.tooltip")} }
                                        }
                                    }
                                }
                            }

                            // Tenant DB migrations
                            Card {
                                CardHeader {
                                    CardTitle { {tid!("developer-actions-database-tenant-migrations")} }
                                    CardDescription { {tid!("developer-actions-database-tenant-migrations.description")} }
                                }
                                CardContent {
                                    Tooltip {
                                        TooltipTrigger {
                                            CardAction {
                                                Button {
                                                    onclick: move |_| async move {
                                                        let _ = api::developer::migrate_tenant_database().await;
                                                    },
                                                    div { class: "flex flex-col items-center",
                                                        Icon {
                                                            fill: "black",
                                                            icon: HiArrowRight,
                                                            style: "color: var(--color-btn-icon)",
                                                        }
                                                        {tid!("developer-actions-database-tenant-migrations-run")}
                                                    }
                                                }
                                            }
                                        }
                                        TooltipContent { side: ContentSide::Bottom,
                                            p { {tid!("developer-actions-database-tenant-migrations-run.tooltip")} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
