use crate::locale::Helper;

pub fn load(h: &mut Helper) {
    // Page titles and headers
    h.add("ui.page_title", "Guess Who");
    h.add("ui.game_header", "Guess Who");
    h.add("ui.new_game", "New game, random identity");
    h.add("ui.new_custom_game", "Game for friend");
    h.add("ui.custom_game_design", "Custom game builder");
    h.add("ui.404", "404");
    h.add("ui.server_error", "ServerError");

    // Game component
    h.add("game.prompt", "I have a hidden identity. Try to guess who I am. Ask your question...");
    h.add("game.instructions_toggle", "Game Instructions");
    h.add("game.rule1", "Only the questions that can be answered with YES or NO are allowed.");
    h.add("game.rule2", "If a question cannot be answered with a simple yes/no, the response will be UNABLE.");
    h.add("game.rule3", "Type: \"I'M LOSER\", I'll reveal my identity and explain my answers.");

    // UI elements
    h.add("ui.send", "Send");
    h.add("ui.game_id", "Game Id: {}");

    // Verdict labels
    h.add("verdict.yes", "Yes");
    h.add("verdict.no", "No");
    h.add("verdict.unable", "Unable");
    h.add("verdict.final", "Final");
    h.add("verdict.na", "N/A");
    h.add("verdict.behave", "Behave");

    // Language selector
    h.add("ui.language", "Language");

    // Confirmation dialog
    h.add("dialog.confirm_language_switch", "Switching languages will end the current game. Do you really want to switch?");
    h.add("dialog.yes", "Yes");
    h.add("dialog.no", "No");

    // Custom game design
    h.add("custom.your_name_label", "Identity:");
    h.add("custom.comment_label", "Comment:");
    h.add("custom.identity_placeholder", "identity to guess");
    h.add("custom.comment", "The game description");

    h.add("custom.cancel_button", "Cancel");
    h.add("custom.create_button", "Create");
}