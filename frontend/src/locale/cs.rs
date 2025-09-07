use crate::locale::Helper;

pub fn load(h: &mut Helper) {
    // Page titles and headers
    h.add("ui.page_title", "Hádej kdo jsem");
    h.add("ui.game_header", "Hádej kdo jsem");
    h.add("ui.new_game", "Nová hra, náhodná identita");
    h.add("ui.new_custom_game", "Hra pro přítele");
    h.add("ui.404", "404");
    h.add("ui.server_error", "Chyba serveru");
    h.add("ui.custom_game_design", "Vyrob si svou hru");

    // Game component
    h.add("game.prompt", "Ptej se!");
    h.add("game.instructions_toggle", "Pravidla hry");
    h.add("game.rule1", "Jsou povoleny pouze otázky, na které lze odpovědět ANO nebo NE.");
    h.add("game.rule2", "Pokud na otázku nelze odpovědět jednoduchým ano/ne, odpověď bude NELZE.");
    h.add("game.rule3", "Napiš: \"KONEC\" a já odhalím svou identitu a vysvětlím své odpovědi.");

    // UI elements
    h.add("ui.send", "Odeslat");
    h.add("ui.game_id", "ID hry: {}");

    // Verdict labels
    h.add("verdict.yes", "Ano");
    h.add("verdict.no", "Ne");
    h.add("verdict.unable", "Nelze");
    h.add("verdict.final", "Konec");
    h.add("verdict.behave", "Nezlob!");
    h.add("verdict.na", "N/A");

    // Language selector
    h.add("ui.language", "Jazyk");

    // Confirmation dialog
    h.add("dialog.confirm_language_switch", "Změna jazyka ukončí aktuální hru. Opravdu mám hru ukončit?");
    h.add("dialog.yes", "Ano");
    h.add("dialog.no", "Ne");

    // Custom game design

    h.add("custom.your_name_label", "Identita:");
    h.add("custom.comment_label", "Komentář:");
    h.add("custom.identity_placeholder", "identita k uhodnutí");
    h.add("custom.comment", "Komentář ke hře");


    h.add("custom.cancel_button", "Zrušit");
    h.add("custom.create_button", "Vytvořit");
}