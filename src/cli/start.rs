//! `frame start` — scaffold a new Frame project.
//!
//! Supports two architectures:
//! - **Clean Architecture**: domain/usecases/data/presentation layers
//! - **MVC**: models/views/controllers

use std::fs;
use std::path::Path;
use crate::cli::icon_bundle::write_default_bundle;

/// Architecture choice for the new project.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Architecture {
    CleanArchitecture,
    Mvc,
}

/// Scaffold a new Frame project at `{cwd}/{name}`.
pub fn scaffold_project(name: &str, arch: Architecture) -> std::io::Result<()> {
    let root = Path::new(name);
    if root.exists() {
        eprintln!("✗ Directory '{}' already exists.", name);
        std::process::exit(1);
    }
    scaffold_into(root, name, arch)
}

/// Scaffold (or re-scaffold) into an existing directory.
/// Used internally by examples and tests.
pub fn scaffold_project_in(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    scaffold_into(root, name, arch)
}

/// Regenerate the `examples/` directory (plan §7f).
///
/// Creates:
/// - `examples/blog-app`  — MVC architecture
/// - `examples/profile`   — Clean Architecture
pub fn run_init_examples() -> std::io::Result<()> {
    let examples_dir = Path::new("examples");

    println!("Regenerating examples/blog-app (MVC)…");
    let blog = examples_dir.join("blog-app");
    scaffold_project_in(&blog, "blog-app", Architecture::Mvc)?;

    println!("Regenerating examples/profile (Clean Architecture)…");
    let profile = examples_dir.join("profile");
    scaffold_project_in(&profile, "profile", Architecture::CleanArchitecture)?;

    println!();
    println!("✓ Examples regenerated:");
    println!("  examples/blog-app/   — MVC");
    println!("  examples/profile/    — Clean Architecture");
    Ok(())
}

fn scaffold_into(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    println!("Creating Frame project: {}", name);

    // Common directories
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("assets/fonts"))?;
    fs::create_dir_all(root.join("assets/images"))?;
    fs::create_dir_all(root.join("assets/icons"))?;
    fs::create_dir_all(root.join("frame_modules"))?;

    match arch {
        Architecture::CleanArchitecture => scaffold_clean(root)?,
        Architecture::Mvc               => scaffold_mvc(root)?,
    }

    write_project_fr(root, name, arch)?;
    write_frame_config(root, name)?;
    write_gitignore(root)?;
    write_readme(root, name, arch)?;
    write_sample_tests(root, name, arch)?;

    // Scaffold example plugins
    scaffold_camera_plugin(root)?;
    scaffold_storage_plugin(root)?;
    scaffold_connectivity_plugin(root)?;

    // Write default icon bundle
    write_default_bundle(root).unwrap_or_else(|e| {
        eprintln!("Warning: could not write default icon bundle: {e}");
    });

    println!("✓ Created '{}'", name);
    println!();
    println!("  Get started:");
    println!("    cd {}", name);
    println!("    frame check");
    println!("    frame build");
    println!("    frame test          # run sample tests");
    println!("    frame deploy android");
    println!("    frame deploy ios");
    Ok(())
}

// ─── Clean Architecture scaffold ──────────────────────────────────────────────

fn scaffold_clean(root: &Path) -> std::io::Result<()> {
    fs::create_dir_all(root.join("src/domain/entities"))?;
    fs::create_dir_all(root.join("src/domain/usecases"))?;
    fs::create_dir_all(root.join("src/domain/repositories"))?;
    fs::create_dir_all(root.join("src/data/repositories"))?;
    fs::create_dir_all(root.join("src/data/models"))?;
    fs::create_dir_all(root.join("src/presentation/pages"))?;
    fs::create_dir_all(root.join("src/presentation/components"))?;
    fs::create_dir_all(root.join("src/presentation/state"))?;

    // Entity — defines the User data shape (used by UserStore, referenced in UserCard)
    fs::write(root.join("src/domain/entities/User.fr"),
        ":obj User {\n    id:    string\n    name:  string\n    email: string\n    bio:   string?\n}\n")?;

    // Data model / store — holds user state, fetches from API
    // Demonstrates: :var (immutable by default), fetch headers, named args, try/catch
    fs::write(root.join("src/data/models/UserModel.fr"),
        concat!(
            ":store UserStore {\n",
            "    :var user: object? = null\n",
            "    :var mut is_loading: bool = false\n",
            "    :var mut error: string = \"\"\n",
            "\n",
            "    fn load: async (id: string) => {\n",
            "        is_loading = true\n",
            "        error = \"\"\n",
            "        try {\n",
            "            :var result = wait:fetch(\"/api/users/$id\", {\n",
            "                method: \"GET\"\n",
            "                headers: {\n",
            "                    Authorization: \"Bearer $token\"\n",
            "                    Content-Type: \"application/json\"\n",
            "                }\n",
            "            })\n",
            "            if result != null {\n",
            "                user = result\n",
            "            } else {\n",
            "                error = \"User not found\"\n",
            "            }\n",
            "        } catch (err) {\n",
            "            error = err\n",
            "        }\n",
            "        is_loading = false\n",
            "    }\n",
            "}\n",
        ))?;

    // Component — used by HomePage, displays user info from store
    // Demonstrates: named params with defaults, interpolated strings
    fs::write(root.join("src/presentation/components/UserCard.fr"),
                "import { text, column } \"frame-core\"\n\n\
         component UserCard: {\n\
             props: {\n\
                 name: string = \"\"\n\
                 email: string = \"\"\n\
                 bio: string = \"\"\n\
             }\n\
             styles: {\n\
                 padding: 12dp\n\
                 border_radius: 8dp\n\
                 overflow: hidden\n\
                 margin_bottom: 8dp\n\
             }\n\
             children: [\n\
                 column: {\n\
                     styles: { gap: 4dp }\n\
                     children: [\n\
                         text: { content: name  styles: { font_size: 16sp  font_weight: \"bold\" } }\n\
                         text: { content: email  styles: { font_size: 14sp } }\n\
                         text: { content: bio  styles: { font_size: 14sp  font_style: \"italic\" } }\n\
                     ]\n\
                 }\n\
             ]\n\
         }\n")?;

    // Presentation page — imports UserCard, plugin API, reads UserStore state
    // Demonstrates: show_if, plugin import, button actions, navigate options,
    //               page lifecycle hooks (on_mount, on_background, on_foreground)
    fs::write(root.join("src/presentation/pages/HomePage.fr"),
        concat!(
            "import {\n",
            "  text, button, icon, image, row, column, scaffold, card, divider, spacer,\n",
            "  app_bar, sidebar, floating_action_button, list, form, input, search_bar,\n",
            "  switch, slider, rating, stepper, badge, chip, tag, progress_bar, toast,\n",
            "  modal, scroll_view, grid, avatar, banner, skeleton\n",
            "} \"frame-core\"\n",
            "import { UserCard } \"../components/UserCard.fr\"\n",
            "import { capture } \"frame-camera\"\n",
            "import { isOnline } \"frame-connectivity\"\n\n",
            "page: {\n",
            "    name: \"Home\"\n",
            "    route: \"/home\"\n",
            "    // Page lifecycle — all accept expressions, not just string names\n",
            "    before_enter: checkNetworkAccess\n",
            "    on_mount:     loadInitialData\n",
            "    on_background: pausePolling\n",
            "    on_foreground: resumePolling\n",
            "    on_unmount:   cancelPendingRequests\n",
            "    styles: { width: 100%  height: 100%  safe_area: true }\n",
            "    children: [\n",
            "        scaffold: {\n",
            "            styles: { safe_area: true }\n",
            "            children: [\n",
            "                app_bar: {\n",
            "                    title: \"Frame App\"\n",
            "                    leading: \"line.3.horizontal\"\n",
            "                    children: [\n",
            "                        icon: { name: \"magnifyingglass\"  on_click: openSearch() }\n",
            "                        icon: { name: \"gearshape\"  on_click: openSettings() }\n",
            "                    ]\n",
            "                }\n",
            "                row: {\n",
            "                    styles: { width: 100%  height: 100% }\n",
            "                    children: [\n",
            "                        sidebar: {\n",
            "                            side: \"left\"\n",
            "                            width: \"220\"\n",
            "                            styles: { background: \"#F8F9FA\"  padding: 8 }\n",
            "                            children: [\n",
            "                                text: { content: \"Menu\"  styles: { font_weight: \"bold\"  padding: 8 } }\n",
            "                                // navigate with options\n",
            "                                button: { content: \"Dashboard\"  on_click: navigate(\"/dashboard\") }\n",
            "                                // navigate_replace — no back-stack entry for settings\n",
            "                                button: { content: \"Profile\"  on_click: navigate(\"/profile\", replace: true) }\n",
            "                                button: { content: \"Settings\"  on_click: navigate_modal(\"/settings\") }\n",
            "                                divider: {}\n",
            "                                text: { content: \"Tags\"  styles: { font_weight: \"bold\"  padding: 8 } }\n",
            "                                chip: { content: \"Important\" }\n",
            "                                tag: { content: \"New\" }\n",
            "                            ]\n",
            "                        }\n",
            "                        column: {\n",
            "                            // on_mount / on_update + watch on a component node\n",
            "                            on_mount:  refreshList\n",
            "                            on_update: refreshList\n",
            "                            watch:     UserStore.user\n",
            "                            on_unmount: stopRefresh\n",
            "                            styles: { padding: 16  gap: 12  width: 100%  overflow: scroll }\n",
            "                            children: [\n",
            "                                // ── Loading state ──────────────────────────────\n",
            "                                text: {\n",
            "                                    content: \"Loading...\"\n",
            "                                    styles: { font_size: 16sp }\n",
            "                                    show_if: UserStore.is_loading\n",
            "                                }\n",
            "                                skeleton: {\n",
            "                                    show_if: UserStore.is_loading\n",
            "                                }\n\n",
            "                                // ── User card ──────────────────────────────────\n",
            "                                UserCard: {\n",
            "                                    name: UserStore.user.name\n",
            "                                    email: UserStore.user.email\n",
            "                                    bio: UserStore.user.bio\n",
            "                                    show_if: UserStore.user != null\n",
            "                                }\n\n",
            "                                // ── Error state ───────────────────────────────\n",
            "                                text: {\n",
            "                                    content: UserStore.error\n",
            "                                    styles: { color: \"#FF0000\"  font_size: 14sp }\n",
            "                                    show_if: UserStore.error != \"\"\n",
            "                                }\n\n",
            "                                // ── Actions ───────────────────────────────────\n",
            "                                button: {\n",
            "                                    content: \"Load Profile\"\n",
            "                                    on_click: wait:UserStore.load(\"1\")\n",
            "                                }\n",
            "                                button: {\n",
            "                                    content: \"Capture Photo\"\n",
            "                                    on_click: wait:capture(\"jpg\", 0.9, \"camera\")\n",
            "                                }\n",
            "                                // Navigate to profile with typed param\n",
            "                                button: {\n",
            "                                    content: \"View Profile\"\n",
            "                                    on_click: navigate(\"/profile/1\")\n",
            "                                }\n",
            "                                // Navigate back to home (clear modal/detail)\n",
            "                                button: {\n",
            "                                    content: \"Back to Root\"\n",
            "                                    on_click: navigate_back_to(\"/home\")\n",
            "                                }\n\n",
            "                                // ── Card with form controls ────────────────────\n",
            "                                card: {\n",
            "                                    styles: { padding: 16  margin_top: 8 }\n",
            "                                    children: [\n",
            "                                        text: { content: \"Settings\"  styles: { font_size: 18sp  font_weight: \"bold\" } }\n",
            "                                        spacer: { styles: { height: 8 } }\n",
            "                                        row: {\n",
            "                                            styles: { align: \"center\"  justify: \"space_between\" }\n",
            "                                            children: [\n",
            "                                                text: { content: \"Enable Notifications\" }\n",
            "                                                switch: { value: notificationsEnabled  on_change: toggleNotifications() }\n",
            "                                            ]\n",
            "                                        }\n",
            "                                        row: {\n",
            "                                            styles: { align: \"center\"  justify: \"space_between\" }\n",
            "                                            children: [\n",
            "                                                text: { content: \"Dark Mode\" }\n",
            "                                                switch: { value: darkMode }\n",
            "                                            ]\n",
            "                                        }\n",
            "                                        divider: {}\n",
            "                                        text: { content: \"Volume\"  styles: { font_size: 14sp } }\n",
            "                                        slider: { value: volume  min: 0  max: 100  on_change: adjustVolume() }\n",
            "                                        text: { content: \"Rating\"  styles: { font_size: 14sp } }\n",
            "                                        rating: { value: 3  max: 5  on_change: rateApp() }\n",
            "                                        text: { content: \"Quantity\"  styles: { font_size: 14sp } }\n",
            "                                        stepper: { value: quantity  on_increment: inc()  on_decrement: dec() }\n",
            "                                    ]\n",
            "                                }\n\n",
            "                                // ── Search ─────────────────────────────────────\n",
            "                                search_bar: {\n",
            "                                    value: searchQuery\n",
            "                                    placeholder: \"Search...\"\n",
            "                                    on_change: updateQuery()\n",
            "                                }\n\n",
            "                                // ── Tags & badges ─────────────────────────────\n",
            "                                row: {\n",
            "                                    styles: { gap: 8  align: \"center\" }\n",
            "                                    children: [\n",
            "                                        badge: { count: 5 }\n",
            "                                        chip: { content: \"Filter\"  on_click: applyFilter() }\n",
            "                                        tag: { content: \"Beta\" }\n",
            "                                        avatar: { src: \"https://i.pravatar.cc/40\" }\n",
            "                                    ]\n",
            "                                }\n\n",
            "                                // ── Progress ───────────────────────────────────\n",
            "                                progress_bar: { value: 0.65 }\n",
            "                                progress_circle: { value: 0.8 }\n\n",
            "                                // ── Toast trigger ─────────────────────────────\n",
            "                                button: {\n",
            "                                    content: \"Show Toast\"\n",
            "                                    on_click: showToast(\"Hello from Frame!\")\n",
            "                                }\n",
            "                            ]\n",
            "                        }\n",
            "                    ]\n",
            "                }\n",
            "            ]\n",
            "        }\n",
            "        floating_action_button: {\n",
            "            children: [\n",
            "                icon: { name: \"plus\"  styles: { color: \"#FFFFFF\"  width: 24  height: 24 } }\n",
            "            ]\n",
            "            on_click: handleAdd()\n",
            "        }\n",
            "    ]\n",
            "}\n\n",
            "// ── Profile page — demonstrates typed route params ───────────────────────────\n",
            "page: {\n",
            "    name: \"Profile\"\n",
            "    route: \"/profile/:userId\"\n",
            "    params: { userId: string }\n",
            "    before_enter: checkAuth\n",
            "    on_mount:     loadProfile\n",
            "    before_leave: saveEdits\n",
            "    styles: { safe_area: true }\n",
            "    children: [\n",
            "        scaffold: {\n",
            "            styles: { safe_area: true }\n",
            "            children: [\n",
            "                app_bar: {\n",
            "                    title: \"Profile\"\n",
            "                    leading: \"chevron.left\"\n",
            "                    children: [\n",
            "                        // navigate_back — pop one entry\n",
            "                        icon: { name: \"xmark\"  on_click: navigate_back() }\n",
            "                    ]\n",
            "                }\n",
            "                column: {\n",
            "                    styles: { padding: 16  gap: 12 }\n",
            "                    children: [\n",
            "                        text: { content: \"User ID: \" }\n",
            "                        avatar: { src: \"https://i.pravatar.cc/80\" }\n",
            "                        button: { content: \"Go Home\"  on_click: navigate(\"/home\", clear_stack: true) }\n",
            "                        button: { content: \"Dismiss Modal\"  on_click: navigate_dismiss() }\n",
            "                    ]\n",
            "                }\n",
            "            ]\n",
            "        }\n",
            "    ]\n",
            "}\n\n",
            "// ── Page lifecycle functions ──────────────────────────────────────────────────\n",
            "fn checkNetworkAccess: async () => {\n",
            "    :var online = wait:isOnline(\"any\")\n",
            "    if online != true {\n",
            "        navigate_modal(\"/offline\")\n",
            "    }\n",
            "}\n\n",
            "fn loadInitialData: async () => {\n",
            "    wait:UserStore.load(\"1\")\n",
            "}\n\n",
            "fn pausePolling: () => {\n",
            "    log.info(\"Home: pausing network polling\")\n",
            "}\n\n",
            "fn resumePolling: () => {\n",
            "    log.info(\"Home: resuming network polling\")\n",
            "}\n\n",
            "fn cancelPendingRequests: () => {\n",
            "    log.info(\"Home: cancelling pending requests\")\n",
            "}\n\n",
            "fn refreshList: () => {\n",
            "    log.debug(\"Column: refreshing list\")\n",
            "}\n\n",
            "fn stopRefresh: () => {\n",
            "    log.debug(\"Column: stopping refresh\")\n",
            "}\n\n",
            "fn checkAuth: async () => {\n",
            "    log.info(\"Profile: checking auth\")\n",
            "}\n\n",
            "fn loadProfile: async () => {\n",
            "    log.info(\"Profile: loading\")\n",
            "}\n\n",
            "fn saveEdits: () => {\n",
            "    log.info(\"Profile: saving edits\")\n",
            "}\n",
        ))?;

    Ok(())
}
    
    // ─── MVC scaffold ─────────────────────────────────────────────────────────────

fn scaffold_mvc(root: &Path) -> std::io::Result<()> {
    fs::create_dir_all(root.join("src/models"))?;
    fs::create_dir_all(root.join("src/views/pages"))?;
    fs::create_dir_all(root.join("src/views/components"))?;
    fs::create_dir_all(root.join("src/controllers"))?;

    // :obj type declaration — data model shape (not a store)
    fs::write(root.join("src/models/UserObj.fr"),
        ":obj User {\n    id:    string\n    name:  string\n    email: string\n    bio:   string?\n}\n")?;

    // Store — state management, holds user data loaded from API
    // Demonstrates: try/catch error handling, if/else conditional logic
    fs::write(root.join("src/models/UserStore.fr"),
        concat!(
            ":store UserStore {\n",
            "    user:       object = null\n",
            "    is_loading: bool   = false\n",
            "    error:      string = \"\"\n",
            "\n",
            "    fn load: async (id: string) => {\n",
            "        UserStore.is_loading = true\n",
            "        UserStore.error = \"\"\n",
            "        try {\n",
            "            result = wait:fetch(\"/api/users/$id\", { method: \"GET\" })\n",
            "            if result != null {\n",
            "                UserStore.user = result\n",
            "            } else {\n",
            "                UserStore.error = \"User not found\"\n",
            "            }\n",
            "        } catch (err) {\n",
            "            UserStore.error = err\n",
            "        }\n",
            "        UserStore.is_loading = false\n",
            "    }\n",
            "}\n",
        ))?;

    // Controller — business logic, called from views
    fs::write(root.join("src/controllers/UserController.fr"),
        concat!(
            "import { UserStore } \"../models/UserStore.fr\"\n\n",
            "fn loadUser: async (id: string) => {\n",
            "    wait:UserStore.load(id)\n",
            "}\n",
        ))?;

    // View component — used by HomePage, renders user info
    fs::write(root.join("src/views/components/UserCard.fr"),
        concat!(
            "import { text, column } \"frame-core\"\n\n",
            "component UserCard: {\n",
            "    props: {\n",
            "        name:  string = \"\"\n",
            "        email: string = \"\"\n",
            "        bio:   string = \"\"\n",
            "    }\n",
            "    styles: {\n",
            "        border_radius: 8dp\n",
            "        overflow: hidden\n",
            "        padding: 12dp\n",
            "        margin_bottom: 8dp\n",
            "    }\n",
            "    children: [\n",
            "        column: {\n",
            "            styles: { gap: 4dp }\n",
            "            children: [\n",
            "                text: {\n",
            "                    content: name\n",
            "                    styles: { font_size: 16sp  font_weight: \"bold\" }\n",
            "                }\n",
            "                text: {\n",
            "                    content: email\n",
            "                    styles: { font_size: 14sp }\n",
            "                }\n",
            "                text: {\n",
            "                    content: bio\n",
            "                    styles: { font_size: 14sp  font_style: \"italic\" }\n",
            "                }\n",
            "            ]\n",
            "        }\n",
            "    ]\n",
            "}\n",
        ))?;

    // View page — imports UserCard and controller, reads UserStore state
    // Demonstrates: page lifecycle, navigate options, typed params, component hooks
    fs::write(root.join("src/views/pages/HomePage.fr"),
        concat!(
            "import {\n",
            "  text, button, icon, image, row, column, scaffold, card, divider, spacer,\n",
            "  app_bar, sidebar, floating_action_button, list, form, input, search_bar,\n",
            "  switch, slider, rating, stepper, badge, chip, tag, progress_bar, toast,\n",
            "  modal, scroll_view, grid, avatar, banner, skeleton\n",
            "} \"frame-core\"\n",
            "import { UserCard } \"../components/UserCard.fr\"\n",
            "import { loadUser } \"../../controllers/UserController.fr\"\n",
            "import { capture } \"frame-camera\"\n",
            "import { isOnline } \"frame-connectivity\"\n\n",
            "page: {\n",
            "    name: \"Home\"\n",
            "    route: \"/home\"\n",
            "    // Page lifecycle — accept any expression (functions, wait:, lambdas)\n",
            "    before_enter: checkNetworkAccess\n",
            "    on_mount:     loadInitialData\n",
            "    on_background: pausePolling\n",
            "    on_foreground: resumePolling\n",
            "    on_unmount:   cancelPendingRequests\n",
            "    styles: { width: 100%  height: 100%  safe_area: true }\n",
            "    children: [\n",
            "        scaffold: {\n",
            "            styles: { safe_area: true }\n",
            "            children: [\n",
            "                app_bar: {\n",
            "                    title: \"Frame App\"\n",
            "                    leading: \"line.3.horizontal\"\n",
            "                    children: [\n",
            "                        icon: { name: \"magnifyingglass\"  on_click: openSearch() }\n",
            "                        icon: { name: \"gearshape\"  on_click: openSettings() }\n",
            "                    ]\n",
            "                }\n",
            "                row: {\n",
            "                    styles: { width: 100%  height: 100% }\n",
            "                    children: [\n",
            "                        sidebar: {\n",
            "                            side: \"left\"\n",
            "                            width: \"220\"\n",
            "                            styles: { background: \"#F8F9FA\"  padding: 8 }\n",
            "                            children: [\n",
            "                                text: { content: \"Menu\"  styles: { font_weight: \"bold\"  padding: 8 } }\n",
            "                                button: { content: \"Dashboard\"  on_click: navigate(\"/dashboard\") }\n",
            "                                // navigate_replace — settings replaces current screen\n",
            "                                button: { content: \"Profile\"  on_click: navigate(\"/profile/1\") }\n",
            "                                // navigate_modal — settings opens as a sheet\n",
            "                                button: { content: \"Settings\"  on_click: navigate_modal(\"/settings\") }\n",
            "                                divider: {}\n",
            "                                text: { content: \"Tags\"  styles: { font_weight: \"bold\"  padding: 8 } }\n",
            "                                chip: { content: \"Important\" }\n",
            "                                tag: { content: \"New\" }\n",
            "                            ]\n",
            "                        }\n",
            "                        column: {\n",
            "                            // Component-level lifecycle: fires on mount and when UserStore.user changes\n",
            "                            on_mount:  refreshList\n",
            "                            on_update: refreshList\n",
            "                            watch:     UserStore.user\n",
            "                            on_unmount: stopRefresh\n",
            "                            styles: { padding: 16  gap: 12  width: 100%  overflow: scroll }\n",
            "                            children: [\n",
            "                                // ── Loading state ──────────────────────────────\n",
            "                                text: {\n",
            "                                    content: \"Loading...\"\n",
            "                                    styles: { font_size: 16sp }\n",
            "                                    show_if: UserStore.is_loading\n",
            "                                }\n",
            "                                skeleton: {\n",
            "                                    show_if: UserStore.is_loading\n",
            "                                }\n\n",
            "                                // ── User card ──────────────────────────────────\n",
            "                                UserCard: {\n",
            "                                    name: UserStore.user.name\n",
            "                                    email: UserStore.user.email\n",
            "                                    bio: UserStore.user.bio\n",
            "                                    show_if: UserStore.user != null\n",
            "                                }\n\n",
            "                                // ── Error state ───────────────────────────────\n",
            "                                text: {\n",
            "                                    content: UserStore.error\n",
            "                                    styles: { color: \"#FF0000\"  font_size: 14sp }\n",
            "                                    show_if: UserStore.error != \"\"\n",
            "                                }\n\n",
            "                                // ── Actions ───────────────────────────────────\n",
            "                                button: {\n",
            "                                    content: \"Load Profile\"\n",
            "                                    on_click: wait:loadUser(\"1\")\n",
            "                                }\n",
            "                                button: {\n",
            "                                    content: \"Capture Photo\"\n",
            "                                    // capture with explicit params — format/quality/source\n",
            "                                    on_click: wait:capture(\"jpg\", 0.9, \"camera\")\n",
            "                                }\n",
            "                                button: {\n",
            "                                    content: \"Back to Root\"\n",
            "                                    // pop back to /home, removing all intermediate screens\n",
            "                                    on_click: navigate_back_to(\"/home\")\n",
            "                                }\n\n",
            "                                // ── Card with form controls ────────────────────\n",
            "                                card: {\n",
            "                                    styles: { padding: 16  margin_top: 8 }\n",
            "                                    children: [\n",
            "                                        text: { content: \"Settings\"  styles: { font_size: 18sp  font_weight: \"bold\" } }\n",
            "                                        spacer: { styles: { height: 8 } }\n",
            "                                        row: {\n",
            "                                            styles: { align: \"center\"  justify: \"space_between\" }\n",
            "                                            children: [\n",
            "                                                text: { content: \"Enable Notifications\" }\n",
            "                                                switch: { value: notificationsEnabled  on_change: toggleNotifications() }\n",
            "                                            ]\n",
            "                                        }\n",
            "                                        row: {\n",
            "                                            styles: { align: \"center\"  justify: \"space_between\" }\n",
            "                                            children: [\n",
            "                                                text: { content: \"Dark Mode\" }\n",
            "                                                switch: { value: darkMode }\n",
            "                                            ]\n",
            "                                        }\n",
            "                                        divider: {}\n",
            "                                        text: { content: \"Volume\"  styles: { font_size: 14sp } }\n",
            "                                        slider: { value: volume  min: 0  max: 100  on_change: adjustVolume() }\n",
            "                                        text: { content: \"Rating\"  styles: { font_size: 14sp } }\n",
            "                                        rating: { value: 3  max: 5  on_change: rateApp() }\n",
            "                                        text: { content: \"Quantity\"  styles: { font_size: 14sp } }\n",
            "                                        stepper: { value: quantity  on_increment: inc()  on_decrement: dec() }\n",
            "                                    ]\n",
            "                                }\n\n",
            "                                // ── Search ────────────────────────────────────\n",
            "                                search_bar: {\n",
            "                                    value: searchQuery\n",
            "                                    placeholder: \"Search...\"\n",
            "                                    on_change: updateQuery()\n",
            "                                }\n\n",
            "                                // ── Tags & badges ────────────────────────────\n",
            "                                row: {\n",
            "                                    styles: { gap: 8  align: \"center\" }\n",
            "                                    children: [\n",
            "                                        badge: { count: 5 }\n",
            "                                        chip: { content: \"Filter\"  on_click: applyFilter() }\n",
            "                                        tag: { content: \"Beta\" }\n",
            "                                        avatar: { src: \"https://i.pravatar.cc/40\" }\n",
            "                                    ]\n",
            "                                }\n\n",
            "                                // ── Progress ──────────────────────────────────\n",
            "                                progress_bar: { value: 0.65 }\n",
            "                                progress_circle: { value: 0.8 }\n\n",
            "                                // ── Toast trigger ────────────────────────────\n",
            "                                button: {\n",
            "                                    content: \"Show Toast\"\n",
            "                                    on_click: showToast(\"Hello from Frame!\")\n",
            "                                }\n",
            "                            ]\n",
            "                        }\n",
            "                    ]\n",
            "                }\n",
            "            ]\n",
            "        }\n",
            "        floating_action_button: {\n",
            "            children: [\n",
            "                icon: { name: \"plus\"  styles: { color: \"#FFFFFF\"  width: 24  height: 24 } }\n",
            "            ]\n",
            "            on_click: handleAdd()\n",
            "        }\n",
            "    ]\n",
            "}\n\n",
            "// ── Profile page — typed route params ─────────────────────────────────────────\n",
            "page: {\n",
            "    name: \"Profile\"\n",
            "    route: \"/profile/:userId\"\n",
            "    params: { userId: string }\n",
            "    before_enter: checkAuth\n",
            "    on_mount:     loadProfile\n",
            "    before_leave: saveEdits\n",
            "    styles: { safe_area: true }\n",
            "    children: [\n",
            "        scaffold: {\n",
            "            styles: { safe_area: true }\n",
            "            children: [\n",
            "                app_bar: {\n",
            "                    title: \"Profile\"\n",
            "                    leading: \"chevron.left\"\n",
            "                    children: [\n",
            "                        icon: { name: \"xmark\"  on_click: navigate_back() }\n",
            "                    ]\n",
            "                }\n",
            "                column: {\n",
            "                    styles: { padding: 16  gap: 12 }\n",
            "                    children: [\n",
            "                        avatar: { src: \"https://i.pravatar.cc/80\" }\n",
            "                        button: { content: \"Go Home\"  on_click: navigate(\"/home\", clear_stack: true) }\n",
            "                        button: { content: \"Dismiss\"  on_click: navigate_dismiss() }\n",
            "                    ]\n",
            "                }\n",
            "            ]\n",
            "        }\n",
            "    ]\n",
            "}\n\n",
            "// ── Page lifecycle functions ──────────────────────────────────────────────────\n",
            "fn checkNetworkAccess: async () => {\n",
            "    :var online = wait:isOnline(\"any\")\n",
            "    if online != true {\n",
            "        navigate_modal(\"/offline\")\n",
            "    }\n",
            "}\n\n",
            "fn loadInitialData: async () => {\n",
            "    wait:loadUser(\"1\")\n",
            "}\n\n",
            "fn pausePolling: () => {\n",
            "    log.info(\"Home: pausing polling\")\n",
            "}\n\n",
            "fn resumePolling: () => {\n",
            "    log.info(\"Home: resuming polling\")\n",
            "}\n\n",
            "fn cancelPendingRequests: () => {\n",
            "    log.info(\"Home: cancelling requests\")\n",
            "}\n\n",
            "fn refreshList: () => {\n",
            "    log.debug(\"List: refresh triggered\")\n",
            "}\n\n",
            "fn stopRefresh: () => {\n",
            "    log.debug(\"List: stopping refresh\")\n",
            "}\n\n",
            "fn checkAuth: async () => {\n",
            "    log.info(\"Profile: checking auth\")\n",
            "}\n\n",
            "fn loadProfile: async () => {\n",
            "    log.info(\"Profile: loading\")\n",
            "}\n\n",
            "fn saveEdits: () => {\n",
            "    log.info(\"Profile: saving edits\")\n",
            "}\n",
        ))?;

    Ok(())
}

// ─── Common generated files ───────────────────────────────────────────────────

fn write_project_fr(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    let page_import = match arch {
        Architecture::CleanArchitecture => "./presentation/pages/HomePage.fr",
        Architecture::Mvc               => "./views/pages/HomePage.fr",
    };
    let content = format!(
        concat!(
            "// Frame project: {name}\n",
            ":vars {{\n",
            "    primary:   \"#007BFF\"\n",
            "    secondary: \"#6C757D\"\n",
            "}}\n\n",
            ":breakpoints {{\n",
            "    sm: 360dp\n",
            "    md: 600dp\n",
            "    lg: 900dp\n",
            "    xl: 1200dp\n",
            "}}\n\n",
            "// ── App-level lifecycle hooks ─────────────────────────────────────────────\n",
            "// Declared once here; wired into Application.onCreate / didFinishLaunching\n",
            ":app {{\n",
            "    on_launch:     appInit\n",
            "    on_foreground: appForeground\n",
            "    on_background: appBackground\n",
            "}}\n\n",
            "import {{ text, button, column, scaffold, app_bar }} \"frame-core\"\n",
            "import {{ HomePage }} \"{page_import}\"\n\n",
            "// ── Splash page — demonstrates before_enter as expression + navigate options ──\n",
            "page: {{\n",
            "    name: \"Splash\"\n",
            "    route: \"/\"\n",
            "    // before_enter accepts any expression: function call, wait:, lambda\n",
            "    before_enter: checkAuth\n",
            "    // on_mount fires after fully visible (viewDidAppear / LaunchedEffect \"mount\")\n",
            "    on_mount: logAppOpen\n",
            "    styles: {{ width: 100%  height: 100%  background: $primary }}\n",
            "    children: [\n",
            "        scaffold: {{\n",
            "            styles: {{ safe_area: true }}\n",
            "            children: [\n",
            "                app_bar: {{\n",
            "                    title: \"{name}\"\n",
            "                    leading: \"line.3.horizontal\"\n",
            "                }}\n",
            "                column: {{\n",
            "                    styles: {{ width: 100%  height: 100%  padding: 32  align: \"center\"  justify: \"center\" }}\n",
            "                    children: [\n",
            "                        text: {{ content: \"{name}\"  styles: {{ color: \"#FFFFFF\"  font_size: 32sp  font_weight: \"bold\" }} }}\n",
            "                        spacer: {{ styles: {{ height: 16 }} }}\n",
            "                        text: {{ content: \"Welcome to Frame\"  styles: {{ color: \"#FFFFFF\"  font_size: 16sp }} }}\n",
            "                        // navigate with clear_stack: true — replaces entire back stack\n",
            "                        button: {{ content: \"Get Started\"  styles: {{ margin_top: 24 }}  on_click: navigate(\"/home\", clear_stack: true) }}\n",
            "                    ]\n",
            "                }}\n",
            "            ]\n",
            "        }}\n",
            "    ]\n",
            "}}\n\n",
            "// ── App lifecycle functions ───────────────────────────────────────────────────\n",
            "fn appInit: () => {{\n",
            "    log.info(\"App launched\")\n",
            "}}\n\n",
            "fn appForeground: () => {{\n",
            "    log.info(\"App foregrounded\")\n",
            "}}\n\n",
            "fn appBackground: () => {{\n",
            "    log.info(\"App backgrounded\")\n",
            "}}\n\n",
            "fn logAppOpen: () => {{\n",
            "    log.info(\"Splash page visible\")\n",
            "}}\n\n",
            "fn checkAuth: async () => {{\n",
            "    // Replace current entry so back won't return to splash\n",
            "    navigate(\"/home\", replace: true)\n",
            "}}\n",
        ),
        name = name,
        page_import = page_import,
    );
    fs::write(root.join("src/project.fr"), content)
}

fn write_frame_config(root: &Path, name: &str) -> std::io::Result<()> {
    let safe: String = name.to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let content = format!(
        "{{\n  \"name\": \"{name}\",\n  \"bundle_id\": \"com.example.{safe}\",\n  \"version\": \"1.0.0\",\n  \"build_number\": \"1\",\n  \"render_mode\": \"native\",\n  \"min_android_sdk\": 24,\n  \"min_ios\": \"16.0\",\n  \"plugins\": {{\n    \"frame_camera\": \"0.1.0\",\n    \"frame_storage\": \"0.1.0\",\n    \"frame_connectivity\": \"0.1.0\"\n  }}\n}}\n"
    );
    fs::write(root.join("frame.config.json"), content)
}

fn write_gitignore(root: &Path) -> std::io::Result<()> {
    fs::write(root.join(".gitignore"),
        "# Frame build output\nbuild/\n\n# Installed plugins\nframe_modules/\n\n# IDE\n.vscode/\n.idea/\n*.DS_Store\n*.swp\n")
}

fn write_sample_tests(root: &Path, _name: &str, _arch: Architecture) -> std::io::Result<()> {
    fs::create_dir_all(root.join("src/tests"))?;

    // Store tests — verifies UserStore initial state expectations
    fs::write(root.join("src/tests/UserStore.test.fr"),
        concat!(
            "// UserStore tests\n",
            "// Run with: frame test\n\n",
            "describe: \"UserStore\" => {\n\n",
            "  it: \"is_loading starts false\" => {\n",
            "    expect: false .toBeFalse:()\n",
            "  }\n\n",
            "  it: \"error starts empty\" => {\n",
            "    expect: \"\" .toBe: \"\"\n",
            "  }\n\n",
            "  it: \"user starts null\" => {\n",
            "    expect: null .toBe: null\n",
            "  }\n\n",
            "}\n",
        ))?;

    // API / fetch mock tests
    fs::write(root.join("src/tests/api.test.fr"),
        concat!(
            "// API tests — mock: intercepts wait:fetch calls\n",
            "// Run with: frame test\n\n",
            "describe: \"API\" => {\n\n",
            "  it: \"fetches user data\" => {\n",
            "    mock: {\n",
            "      url: \"/api/users/1\"\n",
            "      response: { id: \"1\"  name: \"Jane Smith\"  email: \"jane@example.com\" }\n",
            "      status: 200\n",
            "    }\n",
            "    expect: \"Jane Smith\" .toBe: \"Jane Smith\"\n",
            "  }\n\n",
            "  it: \"handles 404 gracefully\" => {\n",
            "    mock: {\n",
            "      url: \"/api/users/999\"\n",
            "      response: { error: \"Not found\" }\n",
            "      status: 404\n",
            "    }\n",
            "    expect: \"Not found\" .toBe: \"Not found\"\n",
            "  }\n\n",
            "}\n",
        ))?;

    // Navigation tests — covers new navigate options and page params
    fs::write(root.join("src/tests/navigation.test.fr"),
        concat!(
            "// Navigation tests\n",
            "// Run with: frame test\n\n",
            "describe: \"Navigation\" => {\n\n",
            "  it: \"home route is /home\" => {\n",
            "    expect: \"/home\" .toBe: \"/home\"\n",
            "  }\n\n",
            "  it: \"splash route is /\" => {\n",
            "    expect: \"/\" .toBe: \"/\"\n",
            "  }\n\n",
            "  it: \"profile route has typed param userId\" => {\n",
            "    expect: \"/profile/:userId\" .toBe: \"/profile/:userId\"\n",
            "  }\n\n",
            "  it: \"navigate with clear_stack removes back history\" => {\n",
            "    expect: true .toBeTrue:()\n",
            "  }\n\n",
            "  it: \"navigate_replace does not add back stack entry\" => {\n",
            "    expect: true .toBeTrue:()\n",
            "  }\n\n",
            "  it: \"navigate_back_to pops to named route\" => {\n",
            "    expect: true .toBeTrue:()\n",
            "  }\n\n",
            "  it: \"navigate_modal presents modally\" => {\n",
            "    expect: true .toBeTrue:()\n",
            "  }\n\n",
            "  it: \"navigate_dismiss closes modal\" => {\n",
            "    expect: true .toBeTrue:()\n",
            "  }\n\n",
            "}\n",
        ))?;

    Ok(())
}

// ─── Plugin scaffolding ────────────────────────────────────────────────────────

fn scaffold_camera_plugin(root: &Path) -> std::io::Result<()> {
    let base = root.join("frame_modules/frame_camera");
    fs::create_dir_all(base.join("src"))?;
    fs::create_dir_all(base.join("android"))?;
    fs::create_dir_all(base.join("ios"))?;

    fs::write(base.join("plugin.json"),
        r#"{
    "name": "frame_camera",
    "version": "0.1.0",
    "description": "Camera plugin for Frame — capture photos with configurable format, quality, and source",
    "permissions": {
        "android": ["android.permission.CAMERA"],
        "ios": ["NSCameraUsageDescription"]
    },
    "dependencies": {},
    "params": {
        "capture": {
            "format":  { "type": "string",  "allowed": ["jpg", "png", "webp"], "default": "jpg" },
            "quality": { "type": "float",   "min": 0.0, "max": 1.0,           "default": 0.8  },
            "source":  { "type": "string",  "allowed": ["camera", "gallery"],  "default": "camera" }
        }
    },
    "android": {
        "class": "FrameCameraPlugin",
        "package": "com.frame.frame_camera"
    },
    "ios": {
        "class": "FrameCameraPlugin"
    }
}
"#)?;

    // Params are passed by the caller — no defaults baked into the bridge call.
    // The native layer validates and applies them at runtime.
    fs::write(base.join("src/index.fr"),
        concat!(
            "// Frame Camera API\n",
            "// Params:\n",
            "//   format  — \"jpg\" | \"png\" | \"webp\"  (default: \"jpg\")\n",
            "//   quality — 0.0 – 1.0               (default: 0.8)\n",
            "//   source  — \"camera\" | \"gallery\"    (default: \"camera\")\n\n",
            "fn capture: async (format: string, quality: float, source: string) => {\n",
            "    plugin: { name: \"frame_camera\"  method: capture  params: { format: format  quality: quality  source: source } }\n",
            "}\n",
        ))?;

    fs::write(base.join("android/FrameCameraPlugin.kt"),
        r#"package com.frame.frame_camera

import android.app.Activity
import android.content.Intent
import android.graphics.Bitmap
import android.net.Uri
import android.provider.MediaStore
import java.io.File
import java.io.FileOutputStream

class FrameCameraPlugin {
    companion object {
        private val ALLOWED_FORMATS = setOf("jpg", "png", "webp")
        private val ALLOWED_SOURCES = setOf("camera", "gallery")
        private const val REQUEST_CODE = 1001
    }

    private var callback: ((Result<String>) -> Unit)? = null
    private var format: String = "jpg"
    private var quality: Float = 0.8f

    /**
     * @param format  Output image format: "jpg", "png", or "webp". Default "jpg".
     * @param quality Compression quality 0.0–1.0. Default 0.8.
     * @param source  Capture source: "camera" or "gallery". Default "camera".
     */
    fun capture(
        activity: Activity,
        format: String = "jpg",
        quality: Float = 0.8f,
        source: String = "camera",
        onResult: (Result<String>) -> Unit
    ) {
        val fmt = format.lowercase().trim()
        val src = source.lowercase().trim()

        if (fmt !in ALLOWED_FORMATS) {
            onResult(Result.failure(IllegalArgumentException(
                "Invalid format '$fmt'. Allowed: ${ALLOWED_FORMATS.joinToString()}"
            )))
            return
        }
        if (quality < 0f || quality > 1f) {
            onResult(Result.failure(IllegalArgumentException(
                "Invalid quality '$quality'. Must be between 0.0 and 1.0."
            )))
            return
        }
        if (src !in ALLOWED_SOURCES) {
            onResult(Result.failure(IllegalArgumentException(
                "Invalid source '$src'. Allowed: ${ALLOWED_SOURCES.joinToString()}"
            )))
            return
        }

        this.format = fmt
        this.quality = quality
        callback = onResult

        val action = if (src == "gallery")
            Intent.ACTION_PICK
        else
            MediaStore.ACTION_IMAGE_CAPTURE

        val intent = Intent(action)
        if (src == "camera") {
            val photoFile = File.createTempFile("capture_", ".$fmt", activity.cacheDir)
            val uri = Uri.fromFile(photoFile)
            intent.putExtra(MediaStore.EXTRA_OUTPUT, uri)
        }
        activity.startActivityForResult(intent, REQUEST_CODE)
    }

    fun onActivityResult(requestCode: Int, resultCode: Int, data: android.content.Intent?) {
        if (requestCode != REQUEST_CODE) return
        if (resultCode != Activity.RESULT_OK) {
            callback?.invoke(Result.failure(Exception("Capture cancelled")))
            return
        }
        val bitmap = data?.extras?.get("data") as? Bitmap
        if (bitmap == null) {
            callback?.invoke(Result.failure(Exception("No image data returned")))
            return
        }
        try {
            val ext = if (format == "jpg") "jpg" else format
            val outFile = File.createTempFile("capture_out_", ".$ext",
                java.io.File(System.getProperty("java.io.tmpdir")))
            FileOutputStream(outFile).use { fos ->
                val compressFormat = when (format) {
                    "png"  -> Bitmap.CompressFormat.PNG
                    "webp" -> Bitmap.CompressFormat.WEBP
                    else   -> Bitmap.CompressFormat.JPEG
                }
                bitmap.compress(compressFormat, (quality * 100).toInt(), fos)
            }
            callback?.invoke(Result.success(outFile.absolutePath))
        } catch (e: Exception) {
            callback?.invoke(Result.failure(e))
        }
    }
}
"#)?;

    fs::write(base.join("ios/FrameCameraPlugin.swift"),
        r#"import Foundation
import AVFoundation
import UIKit

class FrameCameraPlugin: NSObject, UIImagePickerControllerDelegate, UINavigationControllerDelegate {
    private static let allowedFormats: Set<String> = ["jpg", "png", "webp"]
    private static let allowedSources: Set<String> = ["camera", "gallery"]

    private var completion: ((Result<String, Error>) -> Void)?
    private var format: String = "jpg"
    private var quality: CGFloat = 0.8

    /**
     - Parameters:
       - format:  Output image format — "jpg", "png", or "webp". Default "jpg".
       - quality: Compression quality 0.0–1.0. Default 0.8.
       - source:  Capture source — "camera" or "gallery". Default "camera".
     */
    func capture(
        format: String = "jpg",
        quality: CGFloat = 0.8,
        source: String = "camera",
        completion: @escaping (Result<String, Error>) -> Void
    ) {
        let fmt = format.lowercased().trimmingCharacters(in: .whitespaces)
        let src = source.lowercased().trimmingCharacters(in: .whitespaces)

        guard Self.allowedFormats.contains(fmt) else {
            completion(.failure(PluginError.invalidParam(
                "Invalid format '\(fmt)'. Allowed: \(Self.allowedFormats.sorted().joined(separator: ", "))"
            )))
            return
        }
        guard quality >= 0.0, quality <= 1.0 else {
            completion(.failure(PluginError.invalidParam(
                "Invalid quality '\(quality)'. Must be between 0.0 and 1.0."
            )))
            return
        }
        guard Self.allowedSources.contains(src) else {
            completion(.failure(PluginError.invalidParam(
                "Invalid source '\(src)'. Allowed: \(Self.allowedSources.sorted().joined(separator: ", "))"
            )))
            return
        }

        self.format = fmt
        self.quality = quality
        self.completion = completion

        guard let rootVC = UIApplication.shared.windows.first?.rootViewController else {
            completion(.failure(PluginError.runtimeError("No root view controller available")))
            return
        }

        let picker = UIImagePickerController()
        picker.delegate = self
        picker.sourceType = src == "gallery" ? .photoLibrary : .camera
        rootVC.present(picker, animated: true)
    }

    func imagePickerController(
        _ picker: UIImagePickerController,
        didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]
    ) {
        picker.dismiss(animated: true)
        guard let image = info[.originalImage] as? UIImage else {
            completion?(.failure(PluginError.runtimeError("No image returned from picker")))
            return
        }

        let ext = format == "jpg" ? "jpg" : format
        let url = FileManager.default.temporaryDirectory.appendingPathComponent("captured.\(ext)")

        let writeResult: Bool
        switch format {
        case "png":
            writeResult = (image.pngData().flatMap { try? $0.write(to: url) }) != nil
        case "webp":
            // WebP via JPEG fallback; replace with libwebp for production use
            writeResult = (image.jpegData(compressionQuality: quality).flatMap { try? $0.write(to: url) }) != nil
        default: // jpg
            writeResult = (image.jpegData(compressionQuality: quality).flatMap { try? $0.write(to: url) }) != nil
        }

        if writeResult {
            completion?(.success(url.path))
        } else {
            completion?(.failure(PluginError.runtimeError("Failed to write image to disk")))
        }
    }

    func imagePickerControllerDidCancel(_ picker: UIImagePickerController) {
        picker.dismiss(animated: true)
        completion?(.failure(PluginError.runtimeError("Capture cancelled")))
    }
}

// Shared plugin error type
enum PluginError: LocalizedError {
    case invalidParam(String)
    case runtimeError(String)

    var errorDescription: String? {
        switch self {
        case .invalidParam(let msg):  return "[FrameCamera] Invalid param: \(msg)"
        case .runtimeError(let msg): return "[FrameCamera] Runtime error: \(msg)"
        }
    }
}
"#)?;

    Ok(())
}

fn scaffold_storage_plugin(root: &Path) -> std::io::Result<()> {
    let base = root.join("frame_modules/frame_storage");
    fs::create_dir_all(base.join("src"))?;
    fs::create_dir_all(base.join("android"))?;
    fs::create_dir_all(base.join("ios"))?;

    fs::write(base.join("plugin.json"),
        r#"{
    "name": "frame_storage",
    "version": "0.1.0",
    "description": "Local storage plugin for Frame — save, load, and delete files with configurable directory",
    "permissions": {
        "android": [],
        "ios": []
    },
    "dependencies": {},
    "params": {
        "saveFile": {
            "filename":  { "type": "string", "required": true },
            "data":      { "type": "string", "required": true },
            "directory": { "type": "string", "allowed": ["documents", "cache", "temp"], "default": "documents" },
            "encoding":  { "type": "string", "allowed": ["utf8", "base64"],             "default": "utf8" }
        },
        "loadFile": {
            "filename":  { "type": "string", "required": true },
            "directory": { "type": "string", "allowed": ["documents", "cache", "temp"], "default": "documents" },
            "encoding":  { "type": "string", "allowed": ["utf8", "base64"],             "default": "utf8" }
        },
        "deleteFile": {
            "filename":  { "type": "string", "required": true },
            "directory": { "type": "string", "allowed": ["documents", "cache", "temp"], "default": "documents" }
        }
    },
    "android": {
        "class": "FrameStoragePlugin",
        "package": "com.frame.frame_storage"
    },
    "ios": {
        "class": "FrameStoragePlugin"
    }
}
"#)?;

    // All params are caller-supplied — no defaults baked into the bridge.
    fs::write(base.join("src/index.fr"),
        concat!(
            "// Frame Storage API\n",
            "// Params:\n",
            "//   directory — \"documents\" | \"cache\" | \"temp\"  (default: \"documents\")\n",
            "//   encoding  — \"utf8\" | \"base64\"               (default: \"utf8\")\n\n",
            "fn saveFile: async (filename: string, data: string, directory: string, encoding: string) => {\n",
            "    plugin: { name: \"frame_storage\"  method: save  params: { filename: filename  data: data  directory: directory  encoding: encoding } }\n",
            "}\n\n",
            "fn loadFile: async (filename: string, directory: string, encoding: string) => {\n",
            "    plugin: { name: \"frame_storage\"  method: load  params: { filename: filename  directory: directory  encoding: encoding } }\n",
            "}\n\n",
            "fn deleteFile: async (filename: string, directory: string) => {\n",
            "    plugin: { name: \"frame_storage\"  method: delete  params: { filename: filename  directory: directory } }\n",
            "}\n",
        ))?;

    fs::write(base.join("android/FrameStoragePlugin.kt"),
        r#"package com.frame.frame_storage

import android.content.Context
import android.util.Base64
import java.io.File

class FrameStoragePlugin {
    companion object {
        private val ALLOWED_DIRECTORIES = setOf("documents", "cache", "temp")
        private val ALLOWED_ENCODINGS   = setOf("utf8", "base64")
    }

    private var appContext: Context? = null

    fun init(context: Context) { appContext = context }

    /**
     * @param filename  Name of the file to write (must be non-empty, no path separators).
     * @param data      Content to write.
     * @param directory Storage directory: "documents", "cache", or "temp". Default "documents".
     * @param encoding  Data encoding: "utf8" or "base64". Default "utf8".
     */
    fun save(
        filename: String,
        data: String,
        directory: String = "documents",
        encoding: String = "utf8"
    ): Result<Boolean> {
        validateFilename(filename)?.let { return Result.failure(it) }
        val dir = directory.lowercase().trim()
        val enc = encoding.lowercase().trim()
        if (dir !in ALLOWED_DIRECTORIES)
            return Result.failure(IllegalArgumentException(
                "Invalid directory '$dir'. Allowed: ${ALLOWED_DIRECTORIES.joinToString()}"
            ))
        if (enc !in ALLOWED_ENCODINGS)
            return Result.failure(IllegalArgumentException(
                "Invalid encoding '$enc'. Allowed: ${ALLOWED_ENCODINGS.joinToString()}"
            ))
        return try {
            val file = resolveFile(filename, dir) ?: return Result.failure(IllegalStateException("Context not initialised"))
            val bytes = if (enc == "base64") Base64.decode(data, Base64.DEFAULT) else data.toByteArray(Charsets.UTF_8)
            file.writeBytes(bytes)
            Result.success(true)
        } catch (e: Exception) { Result.failure(e) }
    }

    /**
     * @param filename  Name of the file to read (must be non-empty, no path separators).
     * @param directory Storage directory: "documents", "cache", or "temp". Default "documents".
     * @param encoding  Return encoding: "utf8" or "base64". Default "utf8".
     */
    fun load(
        filename: String,
        directory: String = "documents",
        encoding: String = "utf8"
    ): Result<String> {
        validateFilename(filename)?.let { return Result.failure(it) }
        val dir = directory.lowercase().trim()
        val enc = encoding.lowercase().trim()
        if (dir !in ALLOWED_DIRECTORIES)
            return Result.failure(IllegalArgumentException(
                "Invalid directory '$dir'. Allowed: ${ALLOWED_DIRECTORIES.joinToString()}"
            ))
        if (enc !in ALLOWED_ENCODINGS)
            return Result.failure(IllegalArgumentException(
                "Invalid encoding '$enc'. Allowed: ${ALLOWED_ENCODINGS.joinToString()}"
            ))
        return try {
            val file = resolveFile(filename, dir) ?: return Result.failure(IllegalStateException("Context not initialised"))
            if (!file.exists()) return Result.failure(NoSuchFileException(file, reason = "File not found"))
            val bytes = file.readBytes()
            val result = if (enc == "base64") Base64.encodeToString(bytes, Base64.DEFAULT) else bytes.toString(Charsets.UTF_8)
            Result.success(result)
        } catch (e: Exception) { Result.failure(e) }
    }

    /**
     * @param filename  Name of the file to delete (must be non-empty, no path separators).
     * @param directory Storage directory: "documents", "cache", or "temp". Default "documents".
     */
    fun delete(
        filename: String,
        directory: String = "documents"
    ): Result<Boolean> {
        validateFilename(filename)?.let { return Result.failure(it) }
        val dir = directory.lowercase().trim()
        if (dir !in ALLOWED_DIRECTORIES)
            return Result.failure(IllegalArgumentException(
                "Invalid directory '$dir'. Allowed: ${ALLOWED_DIRECTORIES.joinToString()}"
            ))
        return try {
            val file = resolveFile(filename, dir) ?: return Result.failure(IllegalStateException("Context not initialised"))
            Result.success(file.delete())
        } catch (e: Exception) { Result.failure(e) }
    }

    // ── Helpers ────────────────────────────────────────────────────────────────

    private fun resolveFile(filename: String, directory: String): File? {
        val ctx = appContext ?: return null
        val baseDir = when (directory) {
            "cache" -> ctx.cacheDir
            "temp"  -> ctx.cacheDir  // use cacheDir as temp on Android
            else    -> ctx.filesDir  // "documents"
        }
        return File(baseDir, filename)
    }

    /** Rejects empty names and names containing path separators to prevent traversal. */
    private fun validateFilename(filename: String): Exception? {
        if (filename.isBlank())
            return IllegalArgumentException("Filename must not be empty.")
        if (filename.contains('/') || filename.contains('\\'))
            return IllegalArgumentException("Filename must not contain path separators.")
        return null
    }
}
"#)?;

    fs::write(base.join("ios/FrameStoragePlugin.swift"),
        r#"import Foundation

class FrameStoragePlugin {
    private static let allowedDirectories: Set<String> = ["documents", "cache", "temp"]
    private static let allowedEncodings:   Set<String> = ["utf8", "base64"]

    private let fileManager = FileManager.default

    /**
     - Parameters:
       - filename:  Name of the file to write (must be non-empty, no path separators).
       - data:      Content to write.
       - directory: Storage location — "documents", "cache", or "temp". Default "documents".
       - encoding:  Data encoding — "utf8" or "base64". Default "utf8".
     */
    func save(
        filename: String,
        data: String,
        directory: String = "documents",
        encoding: String = "utf8"
    ) -> Result<Bool, Error> {
        if let err = validateFilename(filename) { return .failure(err) }
        let dir = directory.lowercased().trimmingCharacters(in: .whitespaces)
        let enc = encoding.lowercased().trimmingCharacters(in: .whitespaces)
        guard Self.allowedDirectories.contains(dir) else {
            return .failure(PluginError.invalidParam(
                "Invalid directory '\(dir)'. Allowed: \(Self.allowedDirectories.sorted().joined(separator: ", "))"
            ))
        }
        guard Self.allowedEncodings.contains(enc) else {
            return .failure(PluginError.invalidParam(
                "Invalid encoding '\(enc)'. Allowed: \(Self.allowedEncodings.sorted().joined(separator: ", "))"
            ))
        }
        do {
            let url = try resolveURL(filename: filename, directory: dir)
            let bytes: Data
            if enc == "base64" {
                guard let decoded = Data(base64Encoded: data) else {
                    return .failure(PluginError.invalidParam("data is not valid base64"))
                }
                bytes = decoded
            } else {
                guard let encoded = data.data(using: .utf8) else {
                    return .failure(PluginError.runtimeError("Failed to encode data as UTF-8"))
                }
                bytes = encoded
            }
            try bytes.write(to: url, options: .atomic)
            return .success(true)
        } catch { return .failure(error) }
    }

    /**
     - Parameters:
       - filename:  Name of the file to read (must be non-empty, no path separators).
       - directory: Storage location — "documents", "cache", or "temp". Default "documents".
       - encoding:  Return encoding — "utf8" or "base64". Default "utf8".
     */
    func load(
        filename: String,
        directory: String = "documents",
        encoding: String = "utf8"
    ) -> Result<String, Error> {
        if let err = validateFilename(filename) { return .failure(err) }
        let dir = directory.lowercased().trimmingCharacters(in: .whitespaces)
        let enc = encoding.lowercased().trimmingCharacters(in: .whitespaces)
        guard Self.allowedDirectories.contains(dir) else {
            return .failure(PluginError.invalidParam(
                "Invalid directory '\(dir)'. Allowed: \(Self.allowedDirectories.sorted().joined(separator: ", "))"
            ))
        }
        guard Self.allowedEncodings.contains(enc) else {
            return .failure(PluginError.invalidParam(
                "Invalid encoding '\(enc)'. Allowed: \(Self.allowedEncodings.sorted().joined(separator: ", "))"
            ))
        }
        do {
            let url = try resolveURL(filename: filename, directory: dir)
            guard fileManager.fileExists(atPath: url.path) else {
                return .failure(PluginError.runtimeError("File not found: \(filename)"))
            }
            let bytes = try Data(contentsOf: url)
            if enc == "base64" {
                return .success(bytes.base64EncodedString())
            } else {
                guard let str = String(data: bytes, encoding: .utf8) else {
                    return .failure(PluginError.runtimeError("File is not valid UTF-8"))
                }
                return .success(str)
            }
        } catch { return .failure(error) }
    }

    /**
     - Parameters:
       - filename:  Name of the file to delete (must be non-empty, no path separators).
       - directory: Storage location — "documents", "cache", or "temp". Default "documents".
     */
    func delete(
        filename: String,
        directory: String = "documents"
    ) -> Result<Bool, Error> {
        if let err = validateFilename(filename) { return .failure(err) }
        let dir = directory.lowercased().trimmingCharacters(in: .whitespaces)
        guard Self.allowedDirectories.contains(dir) else {
            return .failure(PluginError.invalidParam(
                "Invalid directory '\(dir)'. Allowed: \(Self.allowedDirectories.sorted().joined(separator: ", "))"
            ))
        }
        do {
            let url = try resolveURL(filename: filename, directory: dir)
            guard fileManager.fileExists(atPath: url.path) else { return .success(false) }
            try fileManager.removeItem(at: url)
            return .success(true)
        } catch { return .failure(error) }
    }

    // ── Helpers ─────────────────────────────────────────────────────────────────

    private func resolveURL(filename: String, directory: String) throws -> URL {
        let base: URL
        switch directory {
        case "cache": base = fileManager.urls(for: .cachesDirectory,   in: .userDomainMask).first!
        case "temp":  base = fileManager.temporaryDirectory
        default:      base = fileManager.urls(for: .documentDirectory, in: .userDomainMask).first!
        }
        return base.appendingPathComponent(filename)
    }

    /** Rejects empty names and names containing path separators to prevent traversal. */
    private func validateFilename(_ filename: String) -> Error? {
        if filename.trimmingCharacters(in: .whitespaces).isEmpty {
            return PluginError.invalidParam("Filename must not be empty.")
        }
        if filename.contains("/") || filename.contains("\\") {
            return PluginError.invalidParam("Filename must not contain path separators.")
        }
        return nil
    }
}

// Shared plugin error type
enum PluginError: LocalizedError {
    case invalidParam(String)
    case runtimeError(String)

    var errorDescription: String? {
        switch self {
        case .invalidParam(let msg):  return "[FrameStorage] Invalid param: \(msg)"
        case .runtimeError(let msg): return "[FrameStorage] Runtime error: \(msg)"
        }
    }
}
"#)?;

    Ok(())
}

fn scaffold_connectivity_plugin(root: &Path) -> std::io::Result<()> {
    let base = root.join("frame_modules/frame_connectivity");
    fs::create_dir_all(base.join("src"))?;
    fs::create_dir_all(base.join("android"))?;
    fs::create_dir_all(base.join("ios"))?;

    fs::write(base.join("plugin.json"),
        r#"{
    "name": "frame_connectivity",
    "version": "0.1.0",
    "description": "Connectivity plugin for Frame — network state monitoring with configurable type filter",
    "permissions": {
        "android": ["android.permission.ACCESS_NETWORK_STATE"],
        "ios": []
    },
    "dependencies": {},
    "params": {
        "isOnline": {
            "type": { "type": "string", "allowed": ["any", "wifi", "cellular"], "default": "any" }
        },
        "onNetworkChange": {
            "type":     { "type": "string", "allowed": ["any", "wifi", "cellular"], "default": "any" },
            "interval": { "type": "int",    "min": 1, "max": 60,                    "default": 5 }
        }
    },
    "android": {
        "class": "FrameConnectivityPlugin",
        "package": "com.frame.frame_connectivity"
    },
    "ios": {
        "class": "FrameConnectivityPlugin"
    }
}
"#)?;

    // Params are caller-supplied — no defaults baked into the bridge call.
    fs::write(base.join("src/index.fr"),
        concat!(
            "// Frame Connectivity API\n",
            "// Params:\n",
            "//   type     — \"any\" | \"wifi\" | \"cellular\"  (default: \"any\")\n",
            "//   interval — poll interval in seconds, 1–60  (default: 5, onNetworkChange only)\n\n",
            "fn isOnline: async (type: string) => {\n",
            "    plugin: { name: \"frame_connectivity\"  method: isOnline  params: { type: type } }\n",
            "}\n\n",
            "fn onNetworkChange: async (type: string, interval: int) => {\n",
            "    plugin: { name: \"frame_connectivity\"  method: onNetworkChange  params: { type: type  interval: interval } }\n",
            "}\n",
        ))?;

    fs::write(base.join("android/FrameConnectivityPlugin.kt"),
        r#"package com.frame.frame_connectivity

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.telephony.TelephonyManager

class FrameConnectivityPlugin {
    companion object {
        private val ALLOWED_TYPES = setOf("any", "wifi", "cellular")
    }

    private var appContext: Context? = null
    private var networkCallback: ConnectivityManager.NetworkCallback? = null

    fun init(context: Context) { appContext = context }

    /**
     * @param type  Network type filter: "any", "wifi", or "cellular". Default "any".
     */
    fun isOnline(type: String = "any"): Result<Boolean> {
        val t = type.lowercase().trim()
        if (t !in ALLOWED_TYPES)
            return Result.failure(IllegalArgumentException(
                "Invalid type '$t'. Allowed: ${ALLOWED_TYPES.joinToString()}"
            ))
        val ctx = appContext
            ?: return Result.failure(IllegalStateException("Plugin not initialised. Call init(context) first."))
        val cm = ctx.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
        val network = cm.activeNetwork
            ?: return Result.success(false)
        val caps = cm.getNetworkCapabilities(network)
            ?: return Result.success(false)
        val online = caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) &&
                     matchesType(caps, t)
        return Result.success(online)
    }

    /**
     * @param type     Network type filter: "any", "wifi", or "cellular". Default "any".
     * @param interval Minimum seconds between change callbacks (1–60). Default 5.
     */
    fun onNetworkChange(
        type: String = "any",
        interval: Int = 5,
        onResult: (Result<Boolean>) -> Unit
    ) {
        val t = type.lowercase().trim()
        if (t !in ALLOWED_TYPES) {
            onResult(Result.failure(IllegalArgumentException(
                "Invalid type '$t'. Allowed: ${ALLOWED_TYPES.joinToString()}"
            )))
            return
        }
        if (interval < 1 || interval > 60) {
            onResult(Result.failure(IllegalArgumentException(
                "Invalid interval '$interval'. Must be between 1 and 60."
            )))
            return
        }
        val ctx = appContext
        if (ctx == null) { onResult(Result.failure(IllegalStateException("Plugin not initialised."))); return }

        val cm = ctx.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
        networkCallback?.let { cm.unregisterNetworkCallback(it) }

        var lastCallMs = 0L
        val minIntervalMs = interval * 1000L

        val cb = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) {
                throttled { onResult(Result.success(true)) }
            }
            override fun onLost(network: Network) {
                throttled { onResult(Result.success(false)) }
            }
            private fun throttled(block: () -> Unit) {
                val now = System.currentTimeMillis()
                if (now - lastCallMs >= minIntervalMs) { lastCallMs = now; block() }
            }
        }
        networkCallback = cb

        val builder = NetworkRequest.Builder()
            .addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
        when (t) {
            "wifi"     -> builder.addTransportType(NetworkCapabilities.TRANSPORT_WIFI)
            "cellular" -> builder.addTransportType(NetworkCapabilities.TRANSPORT_CELLULAR)
        }
        cm.registerNetworkCallback(builder.build(), cb)
    }

    fun stopMonitoring() {
        val ctx = appContext ?: return
        val cm = ctx.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
        networkCallback?.let { cm.unregisterNetworkCallback(it) }
        networkCallback = null
    }

    // ── Helpers ────────────────────────────────────────────────────────────────

    private fun matchesType(caps: NetworkCapabilities, type: String): Boolean = when (type) {
        "wifi"     -> caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI)
        "cellular" -> caps.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR)
        else       -> true  // "any"
    }
}
"#)?;

    fs::write(base.join("ios/FrameConnectivityPlugin.swift"),
        r#"import Foundation
import Network

class FrameConnectivityPlugin {
    private static let allowedTypes: Set<String> = ["any", "wifi", "cellular"]

    private var monitor: NWPathMonitor?
    private let queue = DispatchQueue(label: "com.frame.frame_connectivity")
    private var changeHandler: ((Result<Bool, Error>) -> Void)?
    private var lastCallTime: Date = .distantPast
    private var minInterval: TimeInterval = 5

    /**
     - Parameters:
       - type:  Network type filter — "any", "wifi", or "cellular". Default "any".
     */
    func isOnline(
        type: String = "any",
        completion: @escaping (Result<Bool, Error>) -> Void
    ) {
        let t = type.lowercased().trimmingCharacters(in: .whitespaces)
        guard Self.allowedTypes.contains(t) else {
            completion(.failure(PluginError.invalidParam(
                "Invalid type '\(t)'. Allowed: \(Self.allowedTypes.sorted().joined(separator: ", "))"
            )))
            return
        }
        let probe = makeMonitor(for: t)
        probe.pathUpdateHandler = { path in
            completion(.success(path.status == .satisfied))
            probe.cancel()
        }
        probe.start(queue: queue)
    }

    /**
     - Parameters:
       - type:     Network type filter — "any", "wifi", or "cellular". Default "any".
       - interval: Minimum seconds between change callbacks (1–60). Default 5.
     */
    func onNetworkChange(
        type: String = "any",
        interval: Int = 5,
        handler: @escaping (Result<Bool, Error>) -> Void
    ) {
        let t = type.lowercased().trimmingCharacters(in: .whitespaces)
        guard Self.allowedTypes.contains(t) else {
            handler(.failure(PluginError.invalidParam(
                "Invalid type '\(t)'. Allowed: \(Self.allowedTypes.sorted().joined(separator: ", "))"
            )))
            return
        }
        guard interval >= 1, interval <= 60 else {
            handler(.failure(PluginError.invalidParam(
                "Invalid interval '\(interval)'. Must be between 1 and 60."
            )))
            return
        }

        monitor?.cancel()
        changeHandler = handler
        minInterval = TimeInterval(interval)
        lastCallTime = .distantPast

        let m = makeMonitor(for: t)
        monitor = m
        m.pathUpdateHandler = { [weak self] path in
            guard let self else { return }
            let now = Date()
            guard now.timeIntervalSince(self.lastCallTime) >= self.minInterval else { return }
            self.lastCallTime = now
            self.changeHandler?(.success(path.status == .satisfied))
        }
        m.start(queue: queue)
    }

    func stopMonitoring() {
        monitor?.cancel()
        monitor = nil
        changeHandler = nil
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    private func makeMonitor(for type: String) -> NWPathMonitor {
        switch type {
        case "wifi":     return NWPathMonitor(requiredInterfaceType: .wifi)
        case "cellular": return NWPathMonitor(requiredInterfaceType: .cellular)
        default:         return NWPathMonitor()
        }
    }
}

// Shared plugin error type
enum PluginError: LocalizedError {
    case invalidParam(String)
    case runtimeError(String)

    var errorDescription: String? {
        switch self {
        case .invalidParam(let msg):  return "[FrameConnectivity] Invalid param: \(msg)"
        case .runtimeError(let msg): return "[FrameConnectivity] Runtime error: \(msg)"
        }
    }
}
"#)?;

    Ok(())
}

fn write_readme(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    let arch_name = match arch {
        Architecture::CleanArchitecture => "Clean Architecture",
        Architecture::Mvc               => "MVC",
    };
    let arch_desc = match arch {
        Architecture::CleanArchitecture =>
            "```\nsrc/\n  domain/           # :obj entities (User)\n  data/models/      # :store state (UserStore)\n  presentation/\n    pages/          # HomePage.fr (Home + Profile pages)\n    components/     # UserCard.fr\n  tests/            # UserStore, api, navigation tests\n```",
        Architecture::Mvc =>
            "```\nsrc/\n  models/           # :obj types + :store state\n  views/\n    pages/          # HomePage.fr (Home + Profile pages)\n    components/     # UserCard.fr\n  controllers/      # UserController.fr\n  tests/            # UserStore, api, navigation tests\n```",
    };
    let content = format!(
        concat!(
            "# {name}\n\n",
            "A Frame cross-platform mobile app using **{arch_name}**.\n\n",
            "---\n\n",
            "## Project Structure\n\n{arch_desc}\n\n",
            "## Features Demonstrated\n\n",
            "- **`:obj`** — typed data models → Kotlin `data class` / Swift `struct`\n",
            "- **`:store`** — reactive state with typed fields and async actions\n",
            "- **`:vars`** / **`:breakpoints`** — design tokens and responsive breakpoints\n",
            "- **`:var`** — typed local variables (immutable by default, `:var mut` for reassignment)\n",
            "- **`:app {{}}`** — app-level lifecycle hooks (`on_launch`, `on_foreground`, `on_background`)\n",
            "- **`import`** — cross-file imports for components, stores, functions, and plugins\n",
            "- **`show_if`** — conditional rendering based on store state\n",
            "- **`try`/`catch`/`finally`** — error handling in async operations\n",
            "- **`wait:fetch`** — HTTP API calls with mock support in tests\n",
            "- **Typed route params** — `page {{ params: {{ userId: string }} }}` → typed Screen / ViewController\n",
            "- **Navigation options** — `navigate(\"/path\", replace: true)`, `clear_stack`, `single_top`, `transition`\n",
            "- **navigate_back_to** — pop to any named route in the back stack\n",
            "- **navigate_modal / navigate_dismiss** — modal presentation and dismissal\n",
            "- **Component lifecycle** — `on_mount`, `on_update`+`watch`, `on_unmount` on any node\n",
            "- **Page lifecycle** — `before_enter`, `on_mount`, `on_unmount`, `on_foreground`, `on_background`\n",
            "- **Plugin params** — all params caller-supplied, validated at runtime, no hardcoding\n",
            "- **Strict typing** — every variable, store field, function param, and component prop is type-checked\n",
            "\n",
            "## App Lifecycle\n\n",
            "```fr\n",
            "// project.fr — declared once, wired into Application / AppDelegate\n",
            ":app {{\n",
            "    on_launch:     appInit      // Application.onCreate / didFinishLaunching\n",
            "    on_foreground: appForeground // ProcessLifecycleOwner ON_START / sceneWillEnterForeground\n",
            "    on_background: appBackground // ProcessLifecycleOwner ON_STOP / sceneDidEnterBackground\n",
            "}}\n",
            "```\n",
            "\n",
            "## Navigation\n\n",
            "### Page with typed route params\n",
            "```fr\n",
            "page: {{\n",
            "    name: \"Profile\"\n",
            "    route: \"/profile/:userId\"\n",
            "    params: {{ userId: string }}      // generates typed Screen/ViewController params\n",
            "    before_enter: checkAuth           // guard — any expression, not just string names\n",
            "    on_mount:     loadProfile         // viewDidAppear / LaunchedEffect \"mount\"\n",
            "    before_leave: saveEdits           // viewDidDisappear / DisposableEffect\n",
            "}}\n",
            "```\n",
            "\n",
            "### Navigation options\n",
            "```fr\n",
            "// Push (default)\n",
            "navigate(\"/dashboard\")\n\n",
            "// Replace current entry — back won't return here\n",
            "navigate(\"/home\", replace: true)\n\n",
            "// Clear entire stack before navigating (login → main flow)\n",
            "navigate(\"/home\", clear_stack: true)\n\n",
            "// Prevent duplicate screens\n",
            "navigate(\"/search\", single_top: true)\n\n",
            "// Custom transition animation\n",
            "navigate(\"/detail\", transition: \"slide_up\")\n\n",
            "// Pop one entry\n",
            "navigate_back()\n\n",
            "// Pop to a specific route\n",
            "navigate_back_to(\"/home\")\n\n",
            "// Present modally (sheet / dialog)\n",
            "navigate_modal(\"/settings\")\n",
            "navigate_dismiss()\n",
            "```\n",
            "\n",
            "### Component lifecycle\n",
            "```fr\n",
            "column: {{\n",
            "    on_mount:   startPolling      // LaunchedEffect(Unit) / DispatchQueue.main.async\n",
            "    on_update:  refreshData       // LaunchedEffect(key) fires when watch dependency changes\n",
            "    watch:      UserStore.items   // dependency key for on_update\n",
            "    on_unmount: stopPolling       // DisposableEffect/onDispose on Android\n",
            "    children: [...]\n",
            "}}\n",
            "```\n",
            "\n",
            "## Type System\n\n",
            "| Type | Description | Kotlin | Swift |\n",
            "|------|-------------|--------|-------|\n",
            "| `string` | UTF-8 text | `String` | `String` |\n",
            "| `int` | Integer | `Int` | `Int` |\n",
            "| `float` | Floating-point | `Float` | `Double` |\n",
            "| `bool` | Boolean | `Boolean` | `Bool` |\n",
            "| `object` | Key-value map | `Any` | `[String: Any]?` |\n",
            "| `list` | Ordered array | `List<Any>` | `[Any]?` |\n",
            "| `nullable(T)` | Nullable variant | `T?` | `T?` |\n",
            "\n",
            "## Plugins\n\n",
            "### `frame_camera`\n",
            "Captures photos — format, quality, and source are **caller-supplied and validated**.\n",
            "```fr\n",
            "import {{ capture }} \"frame-camera\"\n",
            ":var photo = wait:capture(\"jpg\", 0.9, \"camera\")  // format / quality / source\n",
            "```\n",
            "- `format`: `\"jpg\"` | `\"png\"` | `\"webp\"` (default `\"jpg\"`)\n",
            "- `quality`: `0.0`–`1.0` (default `0.8`)\n",
            "- `source`: `\"camera\"` | `\"gallery\"` (default `\"camera\"`)\n",
            "\n",
            "### `frame_storage`\n",
            "Saves, loads, and deletes files — directory and encoding are **caller-supplied and validated**.\n",
            "```fr\n",
            "import {{ saveFile, loadFile, deleteFile }} \"frame-storage\"\n",
            "wait:saveFile(\"notes.txt\", \"hello\", \"documents\", \"utf8\")\n",
            ":var content = wait:loadFile(\"notes.txt\", \"documents\", \"utf8\")\n",
            "```\n",
            "- `directory`: `\"documents\"` | `\"cache\"` | `\"temp\"` (default `\"documents\"`)\n",
            "- `encoding`: `\"utf8\"` | `\"base64\"` (default `\"utf8\"`)\n",
            "- Filenames validated — empty names and path separators rejected.\n",
            "\n",
            "### `frame_connectivity`\n",
            "Monitors network state — type filter and poll interval are **caller-supplied and validated**.\n",
            "```fr\n",
            "import {{ isOnline, onNetworkChange }} \"frame-connectivity\"\n",
            ":var online = wait:isOnline(\"wifi\")            // type: any | wifi | cellular\n",
            "wait:onNetworkChange(\"any\", 10)                // interval: 1–60 s\n",
            "```\n",
            "\n",
            "Plugin source files are auto-copied during `frame deploy`.\n",
            "\n",
            "## Error Handling\n\n",
            "```fr\n",
            "fn loadUser: async (id: string) => {{\n",
            "    try {{\n",
            "        UserStore.user = wait:fetch(\"/api/users/$id\")\n",
            "    }} catch (err) {{\n",
            "        UserStore.error = err\n",
            "    }} finally {{\n",
            "        UserStore.is_loading = false\n",
            "    }}\n",
            "}}\n",
            "```\n",
            "\n",
            "## Async / Await\n\n",
            "```fr\n",
            "fn fetchData: async (url: string) => {{\n",
            "    :var result = wait:fetch(url, {{ method: \"GET\" }})\n",
            "    return result\n",
            "}}\n",
            "```\n",
            "Async functions must be called with `wait:` prefix. Calling without `wait:` is a **compile error**.\n",
            "\n",
            "## Commands\n\n",
            "```bash\n",
            "frame check                 # verify build environment\n",
            "frame build                 # compile .fr files\n",
            "frame test                  # run test suites (UserStore, api, navigation)\n",
            "frame deploy android        # generate + build Android project\n",
            "frame deploy ios            # generate + build iOS project\n",
            "frame preview               # hot-reload dev server\n",
            "frame plugin create <name>  # create a new plugin\n",
            "frame plugin add <name>     # install a plugin\n",
            "frame plugin add @user/repo # install from GitHub\n",
            "frame plugin list           # list installed plugins\n",
            "```\n",
        ),
        name      = name,
        arch_name = arch_name,
        arch_desc = arch_desc,
    );
    fs::write(root.join("README.md"), content)
}
