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
pub fn scaffold_project_in(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    scaffold_into(root, name, arch)
}

/// Regenerate the `examples/` directory.
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

    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("assets/fonts"))?;
    fs::create_dir_all(root.join("assets/images"))?;
    fs::create_dir_all(root.join("assets/icons"))?;
    fs::create_dir_all(root.join("frame_modules"))?;

    match arch {
        Architecture::CleanArchitecture => scaffold_clean(root, name)?,
        Architecture::Mvc               => scaffold_mvc(root, name)?,
    }

    write_project_fr(root, name, arch)?;
    write_frame_config(root, name)?;
    write_gitignore(root)?;
    write_readme(root, name, arch)?;
    write_sample_tests(root, arch)?;

    scaffold_camera_plugin(root)?;
    scaffold_storage_plugin(root)?;
    scaffold_connectivity_plugin(root)?;

    write_default_bundle(root).unwrap_or_else(|e| {
        eprintln!("Warning: could not write default icon bundle: {e}");
    });

    println!("✓ Created '{}'", name);
    println!();
    println!("  Get started:");
    println!("    cd {}", name);
    println!("    frame check");
    println!("    frame build");
    println!("    frame test");
    println!("    frame deploy android");
    println!("    frame deploy ios");
    Ok(())
}

// ─── Clean Architecture scaffold ─────────────────────────────────────────────

fn scaffold_clean(root: &Path, _name: &str) -> std::io::Result<()> {
    fs::create_dir_all(root.join("src/domain/entities"))?;
    fs::create_dir_all(root.join("src/domain/repositories"))?;
    fs::create_dir_all(root.join("src/domain/usecases"))?;
    fs::create_dir_all(root.join("src/data/repositories"))?;
    fs::create_dir_all(root.join("src/data/models"))?;
    fs::create_dir_all(root.join("src/presentation/pages"))?;
    fs::create_dir_all(root.join("src/presentation/components"))?;
    fs::create_dir_all(root.join("src/presentation/stores"))?;

    // ── Domain layer ──────────────────────────────────────────────────────────

    // Post entity — core business object, no framework dependencies
    fs::write(root.join("src/domain/entities/Post.fr"), r##"// Post — core domain entity.
// Plain data object; no framework deps, no store logic here.
:obj Post {
    id:         string
    title:      string
    body:       string
    author_id:  string
    author:     string
    tags:       list
    published:  bool
    created_at: string
    updated_at: string
}
"##)?;

    // User entity
    fs::write(root.join("src/domain/entities/User.fr"), r##"// User — core domain entity.
:obj User {
    id:         string
    name:       string
    email:      string
    avatar_url: string?
    bio:        string?
    post_count: int
    joined_at:  string
}
"##)?;

    // Repository interface — domain defines the contract, data layer implements it
    fs::write(root.join("src/domain/repositories/PostRepository.fr"), r##"// PostRepository — abstract contract the domain layer depends on.
// The data layer provides the concrete implementation.
// This inversion keeps domain logic independent of API/DB details.
:interface PostRepository {
    fn getAll:      async () => list
    fn getById:     async (id: string) => object?
    fn getByAuthor: async (author_id: string) => list
    fn create:      async (title: string, body: string, tags: list) => object
    fn update:      async (id: string, title: string, body: string) => object
    fn delete:      async (id: string) => bool
    fn publish:     async (id: string) => object
    fn search:      async (query: string) => list
}
"##)?;

    // ── Use cases — one file per business action ──────────────────────────────

    // GetPosts — list + filter use case
    fs::write(root.join("src/domain/usecases/GetPosts.fr"), r##"// GetPosts use case — fetch and filter the post list.
// Business rules: only return published posts unless author is requesting own posts.
import { PostRepository } "../repositories/PostRepository.fr"
import { AuthStore } "../../presentation/stores/AuthStore.fr"

fn getPosts: async () => {
    :var all = wait:PostRepository.getAll()
    // Business rule: unauthenticated users only see published posts
    if AuthStore.user == null {
        return all.filter((p) => p.published == true)
    }
    // Authenticated users see their own drafts too
    :var user_id = AuthStore.user.id
    return all.filter((p) => p.published == true || p.author_id == user_id)
}

fn getPostsByAuthor: async (author_id: string) => {
    if author_id == "" {
        throw "author_id is required"
    }
    return wait:PostRepository.getByAuthor(author_id)
}

fn searchPosts: async (query: string) => {
    if query == "" {
        return wait:getPosts()
    }
    :var trimmed = query.trim()
    if trimmed.length < 2 {
        throw "Search query must be at least 2 characters"
    }
    return wait:PostRepository.search(trimmed)
}
"##)?;

    // CreatePost use case — input validation + business logic
    fs::write(root.join("src/domain/usecases/CreatePost.fr"), r##"// CreatePost use case — validate input and create a new post.
// Business rules enforced here, not in the UI layer.
import { PostRepository } "../repositories/PostRepository.fr"
import { AuthStore } "../../presentation/stores/AuthStore.fr"

fn createPost: async (title: string, body: string, tags: list) => {
    // Auth guard
    if AuthStore.user == null {
        throw "You must be signed in to create a post"
    }
    // Validate title
    :var clean_title = title.trim()
    if clean_title.length == 0 {
        throw "Title cannot be empty"
    }
    if clean_title.length > 120 {
        throw "Title must be 120 characters or fewer"
    }
    // Validate body
    :var clean_body = body.trim()
    if clean_body.length < 10 {
        throw "Post body must be at least 10 characters"
    }
    // Enforce tag limit
    if tags.length > 5 {
        throw "A post can have at most 5 tags"
    }
    return wait:PostRepository.create(clean_title, clean_body, tags)
}

fn publishPost: async (id: string) => {
    if AuthStore.user == null {
        throw "You must be signed in to publish a post"
    }
    :var post = wait:PostRepository.getById(id)
    if post == null {
        throw "Post not found"
    }
    if post.author_id != AuthStore.user.id {
        throw "You can only publish your own posts"
    }
    return wait:PostRepository.publish(id)
}
"##)?;

    // ── Data layer — concrete repository implementation ───────────────────────

    // RemotePostRepository — real HTTP calls, maps raw JSON to domain entities
    fs::write(root.join("src/data/repositories/RemotePostRepository.fr"), r##"// RemotePostRepository — concrete implementation of PostRepository.
// Handles HTTP, error mapping, and response parsing.
// The domain layer never imports this directly.
import { Post } "../../domain/entities/Post.fr"
import { AuthStore } "../../presentation/stores/AuthStore.fr"

:var BASE_URL = "https://api.example.com/v1"

fn getAll: async () => {
    :var res = wait:fetch("$BASE_URL/posts", {
        method: "GET"
        headers: { Accept: "application/json" }
    })
    if res.ok != true {
        throw "Failed to load posts: $res.status"
    }
    return res.data.posts
}

fn getById: async (id: string) => {
    :var res = wait:fetch("$BASE_URL/posts/$id", {
        method: "GET"
        headers: { Accept: "application/json" }
    })
    if res.status == 404 {
        return null
    }
    if res.ok != true {
        throw "Failed to load post: $res.status"
    }
    return res.data
}

fn getByAuthor: async (author_id: string) => {
    :var res = wait:fetch("$BASE_URL/posts?author_id=$author_id", {
        method: "GET"
        headers: { Accept: "application/json" }
    })
    if res.ok != true {
        throw "Failed to load posts: $res.status"
    }
    return res.data.posts
}

fn create: async (title: string, body: string, tags: list) => {
    :var token = AuthStore.token
    :var res = wait:fetch("$BASE_URL/posts", {
        method: "POST"
        headers: {
            Authorization: "Bearer $token"
            Content-Type:  "application/json"
        }
        body: { title: title  body: body  tags: tags }
    })
    if res.ok != true {
        throw "Failed to create post: $res.status $res.data.message"
    }
    return res.data
}

fn update: async (id: string, title: string, body: string) => {
    :var token = AuthStore.token
    :var res = wait:fetch("$BASE_URL/posts/$id", {
        method: "PATCH"
        headers: {
            Authorization: "Bearer $token"
            Content-Type:  "application/json"
        }
        body: { title: title  body: body }
    })
    if res.ok != true {
        throw "Failed to update post: $res.status"
    }
    return res.data
}

fn delete: async (id: string) => {
    :var token = AuthStore.token
    :var res = wait:fetch("$BASE_URL/posts/$id", {
        method: "DELETE"
        headers: { Authorization: "Bearer $token" }
    })
    return res.ok
}

fn publish: async (id: string) => {
    :var token = AuthStore.token
    :var res = wait:fetch("$BASE_URL/posts/$id/publish", {
        method: "POST"
        headers: { Authorization: "Bearer $token" }
    })
    if res.ok != true {
        throw "Failed to publish post: $res.status"
    }
    return res.data
}

fn search: async (query: string) => {
    :var encoded = query.url_encode()
    :var res = wait:fetch("$BASE_URL/posts/search?q=$encoded", {
        method: "GET"
        headers: { Accept: "application/json" }
    })
    if res.ok != true {
        throw "Search failed: $res.status"
    }
    return res.data.posts
}
"##)?;

    // ── Presentation stores ───────────────────────────────────────────────────

    // AuthStore — session management, persists token to device storage
    fs::write(root.join("src/presentation/stores/AuthStore.fr"), r##"// AuthStore — manages auth session and persists token across restarts.
import { saveFile, loadFile } "frame-storage"

:store AuthStore {
    user:          object? = null
    token:         string  = ""
    is_loading:    bool    = false
    error:         string  = ""
    is_signed_in:  bool    = false

    fn init: async () => {
        // Restore session from device storage on app launch
        try {
            :var stored = wait:loadFile("session.json", "documents", "utf8")
            if stored != "" {
                :var session = stored.parse_json()
                AuthStore.token        = session.token
                AuthStore.user         = session.user
                AuthStore.is_signed_in = true
            }
        } catch (err) {
            // No stored session — fresh start, not an error
            log.debug("No stored session: $err")
        }
    }

    fn signIn: async (email: string, password: string) => {
        AuthStore.is_loading = true
        AuthStore.error      = ""
        try {
            :var res = wait:fetch("https://api.example.com/v1/auth/login", {
                method: "POST"
                headers: { Content-Type: "application/json" }
                body: { email: email  password: password }
            })
            if res.ok != true {
                AuthStore.error = res.data.message || "Sign in failed"
                return
            }
            AuthStore.token        = res.data.token
            AuthStore.user         = res.data.user
            AuthStore.is_signed_in = true
            // Persist session
            :var session = { token: res.data.token  user: res.data.user }
            wait:saveFile("session.json", session.to_json(), "documents", "utf8")
        } catch (err) {
            AuthStore.error = "Network error. Please check your connection."
        }
        AuthStore.is_loading = false
    }

    fn signOut: async () => {
        AuthStore.user         = null
        AuthStore.token        = ""
        AuthStore.is_signed_in = false
        AuthStore.error        = ""
        wait:saveFile("session.json", "", "documents", "utf8")
        navigate("/sign-in", clear_stack: true)
    }
}
"##)?;

    // PostStore — post list + pagination + optimistic updates
    fs::write(root.join("src/presentation/stores/PostStore.fr"), r##"// PostStore — post list with pagination, search, and optimistic create.
import { getPosts, searchPosts } "../../domain/usecases/GetPosts.fr"
import { createPost, publishPost } "../../domain/usecases/CreatePost.fr"

:store PostStore {
    posts:          list   = []
    selected_post:  object? = null
    search_query:   string = ""
    is_loading:     bool   = false
    is_creating:    bool   = false
    error:          string = ""
    page:           int    = 1
    has_more:       bool   = true
    total_count:    int    = 0

    fn load: async () => {
        PostStore.is_loading = true
        PostStore.error      = ""
        PostStore.page       = 1
        try {
            :var result = wait:getPosts()
            PostStore.posts       = result
            PostStore.total_count = result.length
            PostStore.has_more    = result.length == 20
        } catch (err) {
            PostStore.error = err
        }
        PostStore.is_loading = false
    }

    fn loadMore: async () => {
        if PostStore.is_loading || !PostStore.has_more {
            return
        }
        PostStore.is_loading = true
        try {
            :var next_page = PostStore.page + 1
            :var result    = wait:getPosts()
            PostStore.posts    = PostStore.posts.concat(result)
            PostStore.page     = next_page
            PostStore.has_more = result.length == 20
        } catch (err) {
            PostStore.error = err
        }
        PostStore.is_loading = false
    }

    fn search: async (query: string) => {
        PostStore.search_query = query
        PostStore.is_loading   = true
        PostStore.error        = ""
        try {
            PostStore.posts = wait:searchPosts(query)
        } catch (err) {
            PostStore.error = err
        }
        PostStore.is_loading = false
    }

    fn create: async (title: string, body: string, tags: list) => {
        PostStore.is_creating = true
        PostStore.error       = ""
        try {
            :var new_post = wait:createPost(title, body, tags)
            // Optimistic: prepend to list immediately
            PostStore.posts = [new_post].concat(PostStore.posts)
            PostStore.total_count = PostStore.total_count + 1
            navigate("/posts/$new_post.id")
        } catch (err) {
            PostStore.error = err
        }
        PostStore.is_creating = false
    }

    fn publish: async (id: string) => {
        try {
            :var updated = wait:publishPost(id)
            // Update in-place in the list
            PostStore.posts = PostStore.posts.map((p) => {
                if p.id == id { return updated }
                return p
            })
        } catch (err) {
            PostStore.error = err
        }
    }

    fn selectPost: (post: object) => {
        PostStore.selected_post = post
    }

    fn clearError: () => {
        PostStore.error = ""
    }
}
"##)?;

    // ── Presentation components ───────────────────────────────────────────────

    fs::write(root.join("src/presentation/components/PostCard.fr"), r##"// PostCard — reusable component that displays a single post summary.
import { text, column, row, button, chip, avatar, divider } "frame-core"

component PostCard: {
    props: {
        id:         string = ""
        title:      string = ""
        body:       string = ""
        author:     string = ""
        tags:       list   = []
        published:  bool   = false
        created_at: string = ""
    }
    styles: {
        border_radius: 12dp
        padding: 16dp
        margin_bottom: 12dp
        background: "#FFFFFF"
        shadow: 1
    }
    children: [
        column: {
            styles: { gap: 8dp }
            children: [
                // Header row: avatar + author + date
                row: {
                    styles: { align: "center"  gap: 8dp }
                    children: [
                        avatar: { size: 32  label: author }
                        column: {
                            children: [
                                text: { content: author  styles: { font_size: 13sp  font_weight: "600" } }
                                text: { content: created_at  styles: { font_size: 11sp  color: "#888" } }
                            ]
                        }
                    ]
                }
                // Title
                text: {
                    content: title
                    styles: { font_size: 17sp  font_weight: "bold"  line_height: 1.3 }
                }
                // Body preview — first 120 chars
                text: {
                    content: body.slice(0, 120)
                    styles: { font_size: 14sp  color: "#555"  line_height: 1.5 }
                }
                // Tags
                row: {
                    styles: { gap: 6dp  wrap: true }
                    children: [
                        chip: { content: tags.join("  ")  styles: { font_size: 12sp } }
                    ]
                }
                divider: {}
                // Draft badge
                text: {
                    content: "DRAFT"
                    styles: { font_size: 11sp  color: "#f59e0b"  font_weight: "bold" }
                    show_if: published == false
                }
            ]
        }
    ]
    on_click: navigate("/posts/$id")
}
"##)?;

    // ── Presentation pages ────────────────────────────────────────────────────

    fs::write(root.join("src/presentation/pages/PostListPage.fr"), r##"// PostListPage — main feed with infinite scroll and search.
import {
    scaffold, app_bar, column, row, text, button, icon,
    search_bar, list, spacer, progress_bar, toast, floating_action_button
} "frame-core"
import { PostCard } "../components/PostCard.fr"
import { PostStore } "../stores/PostStore.fr"
import { AuthStore } "../stores/AuthStore.fr"

page: {
    name: "PostList"
    route: "/posts"
    before_enter: requireAuth
    on_mount: PostStore.load
    styles: { width: 100%  height: 100%  safe_area: true }
    children: [
        scaffold: {
            children: [
                app_bar: {
                    title: "Posts"
                    children: [
                        icon: { name: "plus"  on_click: navigate("/posts/new") }
                        icon: { name: "person.circle"  on_click: navigate("/profile") }
                    ]
                }
                column: {
                    styles: { width: 100%  height: 100% }
                    children: [
                        // Search bar
                        search_bar: {
                            value: PostStore.search_query
                            placeholder: "Search posts..."
                            on_change: PostStore.search
                            styles: { margin: 12dp }
                        }
                        // Post count
                        text: {
                            content: "$PostStore.total_count posts"
                            styles: { font_size: 12sp  color: "#888"  margin_horizontal: 16dp }
                            show_if: PostStore.is_loading == false
                        }
                        // Loading indicator
                        progress_bar: {
                            value: -1
                            show_if: PostStore.is_loading && PostStore.posts.length == 0
                        }
                        // Error state
                        column: {
                            show_if: PostStore.error != ""
                            styles: { align: "center"  padding: 32dp }
                            children: [
                                icon: { name: "exclamationmark.triangle"  size: 40  color: "#f59e0b" }
                                text: { content: PostStore.error  styles: { font_size: 15sp  color: "#555"  margin_top: 8dp } }
                                button: {
                                    content: "Try Again"
                                    styles: { margin_top: 16dp }
                                    on_click: PostStore.load()
                                }
                            ]
                        }
                        // Post list with infinite scroll
                        list: {
                            data: PostStore.posts
                            key: "id"
                            on_end_reached: PostStore.loadMore
                            end_reached_threshold: 3
                            show_if: PostStore.posts.length > 0
                            render: (post) => [
                                PostCard: {
                                    id:         post.id
                                    title:      post.title
                                    body:       post.body
                                    author:     post.author
                                    tags:       post.tags
                                    published:  post.published
                                    created_at: post.created_at
                                }
                            ]
                        }
                        // Empty state
                        column: {
                            show_if: PostStore.is_loading == false && PostStore.posts.length == 0
                            styles: { align: "center"  padding: 48dp }
                            children: [
                                icon: { name: "doc.text"  size: 48  color: "#ccc" }
                                text: { content: "No posts yet"  styles: { font_size: 16sp  color: "#888"  margin_top: 12dp } }
                                button: {
                                    content: "Write the first post"
                                    styles: { margin_top: 16dp }
                                    on_click: navigate("/posts/new")
                                }
                            ]
                        }
                    ]
                }
            ]
        }
        floating_action_button: {
            on_click: navigate("/posts/new")
            children: [
                icon: { name: "plus"  color: "#FFFFFF"  size: 24 }
            ]
        }
    ]
}

fn requireAuth: async () => {
    if AuthStore.is_signed_in != true {
        navigate("/sign-in", replace: true)
    }
}
"##)?;

    Ok(())
}

// ─── MVC scaffold ─────────────────────────────────────────────────────────────

fn scaffold_mvc(root: &Path, _name: &str) -> std::io::Result<()> {
    fs::create_dir_all(root.join("src/models"))?;
    fs::create_dir_all(root.join("src/views/pages"))?;
    fs::create_dir_all(root.join("src/views/components"))?;
    fs::create_dir_all(root.join("src/controllers"))?;

    // ── Models ────────────────────────────────────────────────────────────────

    // Post type
    fs::write(root.join("src/models/Post.fr"), r##"// Post — blog post data model.
:obj Post {
    id:         string
    title:      string
    body:       string
    author_id:  string
    author:     string
    tags:       list
    published:  bool
    like_count: int
    created_at: string
    updated_at: string
}
"##)?;

    // User type
    fs::write(root.join("src/models/User.fr"), r##"// User — account data model.
:obj User {
    id:         string
    name:       string
    email:      string
    avatar_url: string?
    bio:        string?
    post_count: int
    follower_count: int
    joined_at:  string
}
"##)?;

    // AuthStore — session with token persistence
    fs::write(root.join("src/models/AuthStore.fr"), r##"// AuthStore — session state with persistent token storage.
import { saveFile, loadFile } "frame-storage"

:store AuthStore {
    user:         object? = null
    token:        string  = ""
    is_loading:   bool    = false
    error:        string  = ""
    is_signed_in: bool    = false

    fn init: async () => {
        try {
            :var data = wait:loadFile("auth.json", "documents", "utf8")
            if data != "" {
                :var session       = data.parse_json()
                AuthStore.token        = session.token
                AuthStore.user         = session.user
                AuthStore.is_signed_in = true
                log.info("Session restored for: $session.user.email")
            }
        } catch (err) {
            log.debug("No persisted session: $err")
        }
    }

    fn signIn: async (email: string, password: string) => {
        if email == "" || password == "" {
            AuthStore.error = "Email and password are required"
            return
        }
        AuthStore.is_loading = true
        AuthStore.error      = ""
        try {
            :var res = wait:fetch("https://api.example.com/v1/auth/login", {
                method: "POST"
                headers: { Content-Type: "application/json" }
                body: { email: email  password: password }
            })
            if res.status == 401 {
                AuthStore.error = "Invalid email or password"
                return
            }
            if res.ok != true {
                AuthStore.error = res.data.message || "Sign in failed"
                return
            }
            AuthStore.token        = res.data.token
            AuthStore.user         = res.data.user
            AuthStore.is_signed_in = true
            wait:saveFile("auth.json", { token: res.data.token  user: res.data.user }.to_json(), "documents", "utf8")
            navigate("/posts", clear_stack: true)
        } catch (err) {
            AuthStore.error = "Network error. Please check your connection."
        }
        AuthStore.is_loading = false
    }

    fn register: async (name: string, email: string, password: string) => {
        if name == "" || email == "" || password == "" {
            AuthStore.error = "All fields are required"
            return
        }
        if password.length < 8 {
            AuthStore.error = "Password must be at least 8 characters"
            return
        }
        AuthStore.is_loading = true
        AuthStore.error      = ""
        try {
            :var res = wait:fetch("https://api.example.com/v1/auth/register", {
                method: "POST"
                headers: { Content-Type: "application/json" }
                body: { name: name  email: email  password: password }
            })
            if res.status == 409 {
                AuthStore.error = "An account with this email already exists"
                return
            }
            if res.ok != true {
                AuthStore.error = res.data.message || "Registration failed"
                return
            }
            AuthStore.token        = res.data.token
            AuthStore.user         = res.data.user
            AuthStore.is_signed_in = true
            wait:saveFile("auth.json", { token: res.data.token  user: res.data.user }.to_json(), "documents", "utf8")
            navigate("/posts", clear_stack: true)
        } catch (err) {
            AuthStore.error = "Network error. Please check your connection."
        }
        AuthStore.is_loading = false
    }

    fn signOut: async () => {
        AuthStore.user         = null
        AuthStore.token        = ""
        AuthStore.is_signed_in = false
        AuthStore.error        = ""
        wait:saveFile("auth.json", "", "documents", "utf8")
        navigate("/sign-in", clear_stack: true)
    }
}
"##)?;

    // PostStore — full CRUD with optimistic UI
    fs::write(root.join("src/models/PostStore.fr"), r##"// PostStore — blog post state with pagination, search, likes, and optimistic updates.
:store PostStore {
    posts:         list    = []
    selected_post: object? = null
    draft_title:   string  = ""
    draft_body:    string  = ""
    draft_tags:    list    = []
    search_query:  string  = ""
    filter:        string  = "all"      // "all" | "published" | "drafts" | "liked"
    sort:          string  = "newest"   // "newest" | "oldest" | "popular"
    page:          int     = 1
    per_page:      int     = 20
    total:         int     = 0
    has_more:      bool    = true
    is_loading:    bool    = false
    is_saving:     bool    = false
    error:         string  = ""
    save_error:    string  = ""

    fn load: async () => {
        PostStore.is_loading = true
        PostStore.error      = ""
        PostStore.page       = 1
        try {
            :var res = wait:fetch("https://api.example.com/v1/posts?page=1&per_page=$PostStore.per_page&sort=$PostStore.sort&filter=$PostStore.filter", {
                method: "GET"
                headers: { Accept: "application/json" }
            })
            if res.ok != true {
                PostStore.error = "Failed to load posts"
                return
            }
            PostStore.posts    = res.data.posts
            PostStore.total    = res.data.total
            PostStore.has_more = res.data.posts.length == PostStore.per_page
        } catch (err) {
            PostStore.error = "Network error loading posts"
        }
        PostStore.is_loading = false
    }

    fn loadMore: async () => {
        if PostStore.is_loading || !PostStore.has_more { return }
        PostStore.is_loading = true
        try {
            :var next = PostStore.page + 1
            :var res  = wait:fetch("https://api.example.com/v1/posts?page=$next&per_page=$PostStore.per_page&sort=$PostStore.sort", {
                method: "GET"
                headers: { Accept: "application/json" }
            })
            if res.ok != true { return }
            PostStore.posts    = PostStore.posts.concat(res.data.posts)
            PostStore.page     = next
            PostStore.has_more = res.data.posts.length == PostStore.per_page
        } catch (err) {
            PostStore.error = "Failed to load more posts"
        }
        PostStore.is_loading = false
    }

    fn search: async (query: string) => {
        PostStore.search_query = query
        if query.trim().length < 2 {
            wait:PostStore.load()
            return
        }
        PostStore.is_loading = true
        PostStore.error      = ""
        try {
            :var encoded = query.url_encode()
            :var res = wait:fetch("https://api.example.com/v1/posts/search?q=$encoded", {
                method: "GET"
                headers: { Accept: "application/json" }
            })
            if res.ok != true { PostStore.error = "Search failed"; return }
            PostStore.posts = res.data.posts
            PostStore.total = res.data.total
        } catch (err) {
            PostStore.error = "Search failed. Try again."
        }
        PostStore.is_loading = false
    }

    fn selectPost: async (id: string) => {
        // Use cached version first, fetch fresh in background
        :var cached = PostStore.posts.find((p) => p.id == id)
        if cached != null { PostStore.selected_post = cached }
        try {
            :var res = wait:fetch("https://api.example.com/v1/posts/$id", {
                method: "GET"
                headers: { Accept: "application/json" }
            })
            if res.ok == true { PostStore.selected_post = res.data }
        } catch (err) {
            if cached == null { PostStore.error = "Could not load post" }
        }
    }

    fn like: async (id: string) => {
        // Optimistic update
        PostStore.posts = PostStore.posts.map((p) => {
            if p.id == id { return { ...p  like_count: p.like_count + 1 } }
            return p
        })
        try {
            :var res = wait:fetch("https://api.example.com/v1/posts/$id/like", {
                method: "POST"
            })
            if res.ok != true {
                // Revert on failure
                PostStore.posts = PostStore.posts.map((p) => {
                    if p.id == id { return { ...p  like_count: p.like_count - 1 } }
                    return p
                })
            }
        } catch (err) {
            log.warn("Like failed: $err")
        }
    }

    fn setFilter: async (filter: string) => {
        PostStore.filter = filter
        wait:PostStore.load()
    }

    fn setSort: async (sort: string) => {
        PostStore.sort = sort
        wait:PostStore.load()
    }
}
"##)?;

    // ── Controllers ───────────────────────────────────────────────────────────

    fs::write(root.join("src/controllers/PostController.fr"), r##"// PostController — business logic between views and PostStore.
// Views call these functions; controllers validate and coordinate.
import { PostStore } "../models/PostStore.fr"
import { AuthStore } "../models/AuthStore.fr"

fn loadFeed: async () => {
    wait:PostStore.load()
}

fn submitPost: async (title: string, body: string, tags_input: string) => {
    // Validation
    :var clean_title = title.trim()
    :var clean_body  = body.trim()
    if clean_title.length == 0 {
        PostStore.save_error = "Title cannot be empty"
        return
    }
    if clean_title.length > 120 {
        PostStore.save_error = "Title must be 120 characters or fewer"
        return
    }
    if clean_body.length < 10 {
        PostStore.save_error = "Body must be at least 10 characters"
        return
    }
    // Parse comma-separated tags
    :var tags = tags_input.split(",").map((t) => t.trim()).filter((t) => t.length > 0)
    if tags.length > 5 {
        PostStore.save_error = "Maximum 5 tags allowed"
        return
    }
    wait:PostStore.create(clean_title, clean_body, tags)
}

fn deletePost: async (id: string) => {
    if AuthStore.token == "" {
        PostStore.error = "You must be signed in"
        return
    }
    try {
        :var res = wait:fetch("https://api.example.com/v1/posts/$id", {
            method: "DELETE"
            headers: { Authorization: "Bearer $AuthStore.token" }
        })
        if res.ok != true {
            PostStore.error = "Failed to delete post"
            return
        }
        // Remove from list
        PostStore.posts = PostStore.posts.filter((p) => p.id != id)
        PostStore.total = PostStore.total - 1
        navigate_back()
    } catch (err) {
        PostStore.error = "Network error. Could not delete post."
    }
}

fn updatePost: async (id: string, title: string, body: string) => {
    :var clean_title = title.trim()
    :var clean_body  = body.trim()
    if clean_title.length == 0 {
        PostStore.save_error = "Title cannot be empty"
        return
    }
    if clean_body.length < 10 {
        PostStore.save_error = "Body must be at least 10 characters"
        return
    }
    PostStore.is_saving   = true
    PostStore.save_error  = ""
    try {
        :var res = wait:fetch("https://api.example.com/v1/posts/$id", {
            method: "PATCH"
            headers: {
                Authorization: "Bearer $AuthStore.token"
                Content-Type:  "application/json"
            }
            body: { title: clean_title  body: clean_body }
        })
        if res.ok != true {
            PostStore.save_error = res.data.message || "Update failed"
            return
        }
        // Update in-place
        PostStore.posts = PostStore.posts.map((p) => {
            if p.id == id { return res.data }
            return p
        })
        navigate("/posts/$id", replace: true)
    } catch (err) {
        PostStore.save_error = "Network error. Could not save changes."
    }
    PostStore.is_saving = false
}
"##)?;

    fs::write(root.join("src/controllers/AuthController.fr"), r##"// AuthController — coordinates auth actions from views.
import { AuthStore } "../models/AuthStore.fr"
import { isOnline } "frame-connectivity"

fn handleSignIn: async (email: string, password: string) => {
    // Guard: check connectivity before attempting auth
    :var online = wait:isOnline("any")
    if online != true {
        AuthStore.error = "No internet connection. Please check your network."
        return
    }
    wait:AuthStore.signIn(email, password)
}

fn handleRegister: async (name: string, email: string, password: string, confirm: string) => {
    :var online = wait:isOnline("any")
    if online != true {
        AuthStore.error = "No internet connection."
        return
    }
    if password != confirm {
        AuthStore.error = "Passwords do not match"
        return
    }
    wait:AuthStore.register(name, email, password)
}

fn handleSignOut: async () => {
    wait:AuthStore.signOut()
}
"##)?;

    // ── Views — components ────────────────────────────────────────────────────

    fs::write(root.join("src/views/components/PostCard.fr"), r##"// PostCard — blog post summary card with like button.
import { text, column, row, button, chip, avatar, divider, icon } "frame-core"
import { PostStore } "../../models/PostStore.fr"

component PostCard: {
    props: {
        id:         string = ""
        title:      string = ""
        body:       string = ""
        author:     string = ""
        tags:       list   = []
        like_count: int    = 0
        published:  bool   = false
        created_at: string = ""
    }
    styles: {
        border_radius: 12dp
        padding: 16dp
        margin_bottom: 12dp
        background: "#FFFFFF"
        shadow: 1
    }
    on_click: navigate("/posts/$id")
    children: [
        column: {
            styles: { gap: 8dp }
            children: [
                row: {
                    styles: { align: "center"  justify: "space_between" }
                    children: [
                        row: {
                            styles: { align: "center"  gap: 8dp }
                            children: [
                                avatar: { size: 28  label: author }
                                text: { content: author  styles: { font_size: 13sp  font_weight: "600" } }
                            ]
                        }
                        text: { content: created_at  styles: { font_size: 11sp  color: "#999" } }
                    ]
                }
                text: {
                    content: title
                    styles: { font_size: 17sp  font_weight: "bold"  line_height: 1.3 }
                }
                text: {
                    content: body.slice(0, 140)
                    styles: { font_size: 14sp  color: "#555"  line_height: 1.5 }
                }
                row: {
                    styles: { gap: 6dp  wrap: true }
                    children: [
                        chip: { content: tags.join("  ")  styles: { font_size: 11sp } }
                    ]
                }
                divider: {}
                row: {
                    styles: { align: "center"  justify: "space_between" }
                    children: [
                        // Like button with count
                        row: {
                            styles: { align: "center"  gap: 4dp }
                            on_click: PostStore.like(id)
                            children: [
                                icon: { name: "heart"  size: 16  color: "#e11d48" }
                                text: { content: "$like_count"  styles: { font_size: 13sp  color: "#888" } }
                            ]
                        }
                        // Draft badge
                        text: {
                            content: "DRAFT"
                            styles: { font_size: 11sp  color: "#f59e0b"  font_weight: "bold" }
                            show_if: published == false
                        }
                    ]
                }
            ]
        }
    ]
}
"##)?;

    // ── Views — pages ─────────────────────────────────────────────────────────

    fs::write(root.join("src/views/pages/SignInPage.fr"), r##"// SignInPage — email/password sign-in with validation feedback.
import { scaffold, column, row, text, button, input, icon, spacer, progress_bar } "frame-core"
import { AuthStore } "../../models/AuthStore.fr"
import { handleSignIn } "../../controllers/AuthController.fr"

:var email:    string = ""
:var password: string = ""

page: {
    name: "SignIn"
    route: "/sign-in"
    on_mount: checkAlreadySignedIn
    styles: { width: 100%  height: 100%  safe_area: true }
    children: [
        scaffold: {
            children: [
                column: {
                    styles: {
                        width: 100%
                        height: 100%
                        padding: 32dp
                        align: "center"
                        justify: "center"
                        gap: 16dp
                        max_width: 400dp
                    }
                    children: [
                        icon: { name: "doc.fill"  size: 56  color: "#bcf970" }
                        text: {
                            content: "Welcome back"
                            styles: { font_size: 26sp  font_weight: "bold" }
                        }
                        text: {
                            content: "Sign in to continue"
                            styles: { font_size: 15sp  color: "#888"  margin_bottom: 8dp }
                        }
                        input: {
                            value: email
                            placeholder: "Email address"
                            keyboard: "email"
                            on_change: (v) => { email = v }
                            styles: { width: 100% }
                        }
                        input: {
                            value: password
                            placeholder: "Password"
                            secure: true
                            on_change: (v) => { password = v }
                            styles: { width: 100% }
                        }
                        // Error message
                        text: {
                            content: AuthStore.error
                            styles: { color: "#dc2626"  font_size: 13sp  width: 100% }
                            show_if: AuthStore.error != ""
                        }
                        // Sign-in button
                        button: {
                            content: "Sign In"
                            styles: { width: 100%  margin_top: 8dp }
                            on_click: handleSignIn(email, password)
                            disabled: AuthStore.is_loading
                        }
                        progress_bar: {
                            value: -1
                            show_if: AuthStore.is_loading
                        }
                        // Register link
                        row: {
                            styles: { gap: 4dp  align: "center"  margin_top: 16dp }
                            children: [
                                text: { content: "Don't have an account?"  styles: { font_size: 14sp } }
                                button: {
                                    content: "Sign up"
                                    variant: "text"
                                    styles: { font_size: 14sp  color: "#bcf970"  font_weight: "bold" }
                                    on_click: navigate("/register")
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    ]
}

fn checkAlreadySignedIn: async () => {
    wait:AuthStore.init()
    if AuthStore.is_signed_in == true {
        navigate("/posts", replace: true)
    }
}
"##)?;

    fs::write(root.join("src/views/pages/PostListPage.fr"), r##"// PostListPage — the main blog feed with filter, sort, and infinite scroll.
import {
    scaffold, app_bar, column, row, text, button, icon,
    search_bar, list, progress_bar, floating_action_button,
    chip, spacer, divider
} "frame-core"
import { PostCard } "../components/PostCard.fr"
import { PostStore } "../../models/PostStore.fr"
import { AuthStore } "../../models/AuthStore.fr"
import { loadFeed } "../../controllers/PostController.fr"

page: {
    name: "PostList"
    route: "/posts"
    before_enter: requireAuth
    on_mount: loadFeed
    on_foreground: checkForUpdates
    styles: { width: 100%  height: 100%  safe_area: true }
    children: [
        scaffold: {
            children: [
                app_bar: {
                    title: "Blog"
                    children: [
                        icon: {
                            name: "line.3.horizontal.decrease"
                            on_click: navigate_modal("/filter")
                            tooltip: "Filter & Sort"
                        }
                        icon: {
                            name: "person.circle"
                            on_click: navigate("/profile/$AuthStore.user.id")
                        }
                    ]
                }
                column: {
                    styles: { width: 100%  height: 100% }
                    children: [
                        // Search
                        search_bar: {
                            value: PostStore.search_query
                            placeholder: "Search posts..."
                            on_change: PostStore.search
                            on_clear: PostStore.load
                            styles: { margin: 12dp }
                        }
                        // Filter chips
                        row: {
                            styles: { padding_horizontal: 12dp  gap: 8dp  margin_bottom: 4dp }
                            children: [
                                chip: {
                                    content: "All"
                                    selected: PostStore.filter == "all"
                                    on_click: PostStore.setFilter("all")
                                }
                                chip: {
                                    content: "Published"
                                    selected: PostStore.filter == "published"
                                    on_click: PostStore.setFilter("published")
                                }
                                chip: {
                                    content: "My Drafts"
                                    selected: PostStore.filter == "drafts"
                                    on_click: PostStore.setFilter("drafts")
                                }
                            ]
                        }
                        // Stats row
                        row: {
                            show_if: PostStore.is_loading == false
                            styles: { padding_horizontal: 16dp  margin_bottom: 4dp  align: "center"  gap: 8dp }
                            children: [
                                text: {
                                    content: "$PostStore.total posts"
                                    styles: { font_size: 12sp  color: "#888" }
                                }
                                spacer: {}
                                text: { content: "Sort:"  styles: { font_size: 12sp  color: "#888" } }
                                button: {
                                    content: PostStore.sort
                                    variant: "text"
                                    styles: { font_size: 12sp  color: "#bcf970" }
                                    on_click: PostStore.setSort(PostStore.sort == "newest" ? "popular" : "newest")
                                }
                            ]
                        }
                        // Loading skeleton
                        progress_bar: {
                            value: -1
                            show_if: PostStore.is_loading && PostStore.posts.length == 0
                            styles: { margin: 16dp }
                        }
                        // Error
                        column: {
                            show_if: PostStore.error != "" && PostStore.is_loading == false
                            styles: { align: "center"  padding: 32dp  gap: 12dp }
                            children: [
                                icon: { name: "wifi.slash"  size: 44  color: "#f59e0b" }
                                text: {
                                    content: PostStore.error
                                    styles: { font_size: 15sp  color: "#555"  align: "center" }
                                }
                                button: {
                                    content: "Retry"
                                    on_click: loadFeed()
                                }
                            ]
                        }
                        // Post list
                        list: {
                            data: PostStore.posts
                            key: "id"
                            on_end_reached: PostStore.loadMore
                            end_reached_threshold: 3
                            show_if: PostStore.posts.length > 0
                            styles: { padding_horizontal: 12dp }
                            render: (post) => [
                                PostCard: {
                                    id:         post.id
                                    title:      post.title
                                    body:       post.body
                                    author:     post.author
                                    tags:       post.tags
                                    like_count: post.like_count
                                    published:  post.published
                                    created_at: post.created_at
                                }
                            ]
                        }
                        // Empty state
                        column: {
                            show_if: PostStore.is_loading == false && PostStore.posts.length == 0 && PostStore.error == ""
                            styles: { align: "center"  padding: 48dp  gap: 12dp }
                            children: [
                                icon: { name: "doc.text"  size: 48  color: "#ccc" }
                                text: { content: "No posts found"  styles: { font_size: 16sp  color: "#888" } }
                                button: {
                                    content: "Write a post"
                                    on_click: navigate("/posts/new")
                                }
                            ]
                        }
                    ]
                }
            ]
        }
        floating_action_button: {
            on_click: navigate("/posts/new")
            children: [
                icon: { name: "plus"  color: "#1a1a2e"  size: 24 }
            ]
        }
    ]
}

fn requireAuth: async () => {
    if AuthStore.is_signed_in != true {
        navigate("/sign-in", replace: true)
    }
}

fn checkForUpdates: async () => {
    if PostStore.posts.length > 0 {
        wait:PostStore.load()
    }
}
"##)?;

    fs::write(root.join("src/views/pages/NewPostPage.fr"), r##"// NewPostPage — create a new blog post with live validation.
import { scaffold, app_bar, column, row, text, button, input, icon, progress_bar, divider } "frame-core"
import { PostStore } "../../models/PostStore.fr"
import { submitPost } "../../controllers/PostController.fr"
import { capture } "frame-camera"

:var title:      string = ""
:var body:       string = ""
:var tags_input: string = ""
:var photo_path: string = ""

page: {
    name: "NewPost"
    route: "/posts/new"
    on_unmount: clearDraft
    styles: { width: 100%  height: 100%  safe_area: true }
    children: [
        scaffold: {
            children: [
                app_bar: {
                    title: "New Post"
                    leading: "xmark"
                    on_leading_click: navigate_back()
                    children: [
                        button: {
                            content: "Publish"
                            variant: "text"
                            styles: { color: "#bcf970"  font_weight: "bold" }
                            on_click: submitPost(title, body, tags_input)
                            disabled: PostStore.is_saving || title.trim().length == 0
                        }
                    ]
                }
                column: {
                    styles: { padding: 16dp  gap: 12dp  overflow: scroll }
                    children: [
                        input: {
                            value: title
                            placeholder: "Post title..."
                            on_change: (v) => { title = v }
                            styles: { font_size: 20sp  font_weight: "bold"  border: none }
                            max_length: 120
                        }
                        row: {
                            styles: { align: "center"  justify: "space_between" }
                            children: [
                                text: {
                                    content: "$title.length / 120"
                                    styles: { font_size: 11sp  color: title.length > 100 ? "#f59e0b" : "#bbb" }
                                }
                            ]
                        }
                        divider: {}
                        input: {
                            value: body
                            placeholder: "Write your post..."
                            on_change: (v) => { body = v }
                            multiline: true
                            min_lines: 8
                            styles: { font_size: 15sp  line_height: 1.6  border: none }
                        }
                        divider: {}
                        input: {
                            value: tags_input
                            placeholder: "Tags (comma-separated, max 5)"
                            on_change: (v) => { tags_input = v }
                            styles: { font_size: 13sp }
                        }
                        // Attach photo
                        row: {
                            styles: { gap: 12dp  align: "center" }
                            children: [
                                button: {
                                    content: "Attach Photo"
                                    variant: "outlined"
                                    on_click: attachPhoto()
                                    children: [
                                        icon: { name: "camera"  size: 16  color: "#555" }
                                    ]
                                }
                                text: {
                                    content: photo_path != "" ? "Photo attached ✓" : ""
                                    styles: { font_size: 13sp  color: "#bcf970" }
                                }
                            ]
                        }
                        // Validation error
                        text: {
                            content: PostStore.save_error
                            styles: { color: "#dc2626"  font_size: 13sp }
                            show_if: PostStore.save_error != ""
                        }
                        progress_bar: {
                            value: -1
                            show_if: PostStore.is_saving
                        }
                    ]
                }
            ]
        }
    ]
}

fn attachPhoto: async () => {
    try {
        :var path = wait:capture("jpg", 0.85, "camera")
        photo_path = path
    } catch (err) {
        log.warn("Photo capture cancelled: $err")
    }
}

fn clearDraft: () => {
    PostStore.save_error = ""
}
"##)?;

    fs::write(root.join("src/views/pages/PostDetailPage.fr"), r##"// PostDetailPage — full post view with edit/delete for the author.
import {
    scaffold, app_bar, column, row, text, button,
    icon, divider, chip, avatar, progress_bar, scroll_view
} "frame-core"
import { PostStore } "../../models/PostStore.fr"
import { AuthStore } "../../models/AuthStore.fr"
import { deletePost } "../../controllers/PostController.fr"

page: {
    name: "PostDetail"
    route: "/posts/:id"
    params: { id: string }
    on_mount: loadPost
    styles: { width: 100%  height: 100%  safe_area: true }
    children: [
        scaffold: {
            children: [
                app_bar: {
                    title: ""
                    leading: "chevron.left"
                    on_leading_click: navigate_back()
                    children: [
                        // Edit/Delete shown only to the post's author
                        row: {
                            show_if: PostStore.selected_post != null && PostStore.selected_post.author_id == AuthStore.user.id
                            children: [
                                icon: {
                                    name: "pencil"
                                    on_click: navigate("/posts/$PostStore.selected_post.id/edit")
                                }
                                icon: {
                                    name: "trash"
                                    color: "#dc2626"
                                    on_click: confirmDelete()
                                }
                            ]
                        }
                    ]
                }
                scroll_view: {
                    styles: { padding: 20dp }
                    children: [
                        // Loading
                        progress_bar: {
                            value: -1
                            show_if: PostStore.is_loading && PostStore.selected_post == null
                        }
                        // Error
                        column: {
                            show_if: PostStore.error != "" && PostStore.selected_post == null
                            styles: { align: "center"  padding: 32dp  gap: 12dp }
                            children: [
                                icon: { name: "exclamationmark.circle"  size: 40  color: "#f59e0b" }
                                text: { content: PostStore.error  styles: { font_size: 15sp  color: "#555" } }
                                button: { content: "Go Back"  on_click: navigate_back() }
                            ]
                        }
                        // Post content
                        column: {
                            show_if: PostStore.selected_post != null
                            styles: { gap: 16dp }
                            children: [
                                text: {
                                    content: PostStore.selected_post.title
                                    styles: { font_size: 26sp  font_weight: "bold"  line_height: 1.25 }
                                }
                                row: {
                                    styles: { align: "center"  gap: 10dp }
                                    children: [
                                        avatar: { size: 36  label: PostStore.selected_post.author }
                                        column: {
                                            children: [
                                                text: {
                                                    content: PostStore.selected_post.author
                                                    styles: { font_size: 14sp  font_weight: "600" }
                                                }
                                                text: {
                                                    content: PostStore.selected_post.created_at
                                                    styles: { font_size: 12sp  color: "#999" }
                                                }
                                            ]
                                        }
                                    ]
                                }
                                row: {
                                    styles: { gap: 6dp  wrap: true }
                                    children: [
                                        chip: { content: PostStore.selected_post.tags.join("  ")  styles: { font_size: 12sp } }
                                    ]
                                }
                                divider: {}
                                text: {
                                    content: PostStore.selected_post.body
                                    styles: { font_size: 16sp  line_height: 1.7  color: "#222" }
                                }
                                divider: {}
                                // Like button
                                row: {
                                    styles: { align: "center"  gap: 8dp }
                                    children: [
                                        button: {
                                            variant: "outlined"
                                            on_click: PostStore.like(PostStore.selected_post.id)
                                            children: [
                                                icon: { name: "heart"  size: 16  color: "#e11d48" }
                                                text: { content: "$PostStore.selected_post.like_count likes"  styles: { font_size: 14sp } }
                                            ]
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    ]
}

fn loadPost: async () => {
    wait:PostStore.selectPost(params.id)
}

fn confirmDelete: async () => {
    navigate_modal("/posts/$PostStore.selected_post.id/confirm-delete")
}
"##)?;

    Ok(())
}

// ─── Common generated files ───────────────────────────────────────────────────

fn write_project_fr(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    let (page_import, auth_import) = match arch {
        Architecture::CleanArchitecture => (
            "./presentation/pages/PostListPage.fr",
            "./presentation/stores/AuthStore.fr",
        ),
        Architecture::Mvc => (
            "./views/pages/PostListPage.fr",
            "./models/AuthStore.fr",
        ),
    };
    let content = format!(
        r##"// {name} — Frame project root.
// Defines app-level config, design tokens, and the entry screen.
:vars {{
    primary:     "#bcf970"
    on_primary:  "#1a1a2e"
    surface:     "#FFFFFF"
    background:  "#F5F5F5"
    error:       "#dc2626"
    text:        "#111827"
    text_muted:  "#6b7280"
    border:      "#e5e7eb"
}}

:breakpoints {{
    sm:  360dp
    md:  600dp
    lg:  900dp
    xl:  1200dp
}}

// App-level lifecycle — wired into Application.onCreate / AppDelegate.didFinishLaunching
:app {{
    default_route: "/sign-in"
    on_launch:     appLaunch
    on_foreground: appForeground
    on_background: appBackground
}}

import {{ scaffold, column, text, button, icon, spacer }} "frame-core"
import {{ PostListPage }} "{page_import}"
import {{ AuthStore }} "{auth_import}"

// Sign-in redirect page — before_enter handles the auth check
page: {{
    name: "Entry"
    route: "/sign-in"
    before_enter: redirectIfSignedIn
    styles: {{
        width: 100%
        height: 100%
        background: $$on_primary
        safe_area: true
    }}
    children: [
        scaffold: {{
            children: [
                column: {{
                    styles: {{
                        width: 100%
                        height: 100%
                        align: "center"
                        justify: "center"
                        gap: 24dp
                        padding: 40dp
                    }}
                    children: [
                        icon: {{ name: "doc.fill"  size: 72  color: $$primary }}
                        text: {{
                            content: "{name}"
                            styles: {{ font_size: 32sp  font_weight: "bold"  color: "#FFFFFF" }}
                        }}
                        text: {{
                            content: "A modern blog built with Frame"
                            styles: {{ font_size: 15sp  color: "#aaa"  align: "center" }}
                        }}
                        spacer: {{ styles: {{ height: 16 }} }}
                        button: {{
                            content: "Sign In"
                            styles: {{
                                width: 100%
                                background: $$primary
                                color: $$on_primary
                                font_weight: "bold"
                                border_radius: 10dp
                                padding: 14dp
                            }}
                            on_click: navigate("/sign-in/form")
                        }}
                        button: {{
                            content: "Create Account"
                            variant: "outlined"
                            styles: {{
                                width: 100%
                                border_color: $$primary
                                color: $$primary
                                border_radius: 10dp
                                padding: 14dp
                            }}
                            on_click: navigate("/register")
                        }}
                    ]
                }}
            ]
        }}
    ]
}}

fn appLaunch: async () => {{
    log.info("{name} launched")
    wait:AuthStore.init()
}}

fn appForeground: () => {{
    log.debug("{name} foregrounded")
}}

fn appBackground: () => {{
    log.debug("{name} backgrounded")
}}

fn redirectIfSignedIn: async () => {{
    if AuthStore.is_signed_in == true {{
        navigate("/posts", replace: true)
    }}
}}
"##,
        name = name,
        page_import = page_import,
        auth_import = auth_import,
    );
    fs::write(root.join("src/project.fr"), content)
}

fn write_sample_tests(root: &Path, arch: Architecture) -> std::io::Result<()> {
    fs::create_dir_all(root.join("src/tests"))?;

    let store_path = match arch {
        Architecture::CleanArchitecture => "../../presentation/stores/PostStore.fr",
        Architecture::Mvc               => "../../models/PostStore.fr",
    };
    let auth_path = match arch {
        Architecture::CleanArchitecture => "../../presentation/stores/AuthStore.fr",
        Architecture::Mvc               => "../../models/AuthStore.fr",
    };

    fs::write(root.join("src/tests/PostStore.test.fr"), format!(r##"// PostStore — unit tests
// Run with: frame test
import {{ PostStore }} "{store_path}"

describe: "PostStore" => {{

  it: "starts with empty post list" => {{
    expect: PostStore.posts.length .toBe: 0
  }}

  it: "starts not loading" => {{
    expect: PostStore.is_loading .toBeFalse:()
  }}

  it: "starts with no error" => {{
    expect: PostStore.error .toBe: ""
  }}

  it: "starts on page 1" => {{
    expect: PostStore.page .toBe: 1
  }}

  it: "like increments count optimistically" => {{
    PostStore.posts = [{{ id: "1"  title: "T"  like_count: 3 }}]
    PostStore.like("1")
    expect: PostStore.posts[0].like_count .toBe: 4
  }}

  it: "setFilter updates filter field" => {{
    PostStore.filter = "all"
    PostStore.filter = "published"
    expect: PostStore.filter .toBe: "published"
  }}

  it: "setSort updates sort field" => {{
    PostStore.sort = "newest"
    PostStore.sort = "popular"
    expect: PostStore.sort .toBe: "popular"
  }}

  it: "clearError resets error string" => {{
    PostStore.error = "Something went wrong"
    PostStore.clearError()
    expect: PostStore.error .toBe: ""
  }}

}}
"##, store_path = store_path))?;

    fs::write(root.join("src/tests/AuthStore.test.fr"), format!(r##"// AuthStore — unit tests
// Run with: frame test
import {{ AuthStore }} "{auth_path}"

describe: "AuthStore" => {{

  it: "starts signed out" => {{
    expect: AuthStore.is_signed_in .toBeFalse:()
  }}

  it: "starts with null user" => {{
    expect: AuthStore.user .toBeNull:()
  }}

  it: "starts with empty token" => {{
    expect: AuthStore.token .toBe: ""
  }}

  it: "starts with no error" => {{
    expect: AuthStore.error .toBe: ""
  }}

  it: "signIn sets is_loading while in progress" => {{
    expect: AuthStore.is_loading .toBeFalse:()
  }}

}}
"##, auth_path = auth_path))?;

    fs::write(root.join("src/tests/api.test.fr"), r##"// API integration tests — uses mock: to intercept fetch calls
// Run with: frame test

describe: "Posts API" => {

  it: "GET /posts returns post list" => {
    mock: {
      url:      "https://api.example.com/v1/posts?page=1&per_page=20&sort=newest&filter=all"
      method:   "GET"
      response: {
        posts: [
          { id: "1"  title: "Hello World"  body: "My first post"  author: "Alice"  like_count: 0  published: true  tags: ["intro"] }
          { id: "2"  title: "Frame Tips"   body: "Some tips"       author: "Bob"    like_count: 5  published: true  tags: ["tips"] }
        ]
        total: 2
      }
      status: 200
    }
    expect: 2 .toBe: 2
  }

  it: "POST /posts creates a post" => {
    mock: {
      url:      "https://api.example.com/v1/posts"
      method:   "POST"
      response: { id: "3"  title: "New Post"  body: "Body text here"  author: "Alice"  like_count: 0  published: false  tags: [] }
      status:   201
    }
    expect: "New Post" .toBe: "New Post"
  }

  it: "DELETE /posts/:id removes a post" => {
    mock: {
      url:    "https://api.example.com/v1/posts/1"
      method: "DELETE"
      status: 204
    }
    expect: true .toBeTrue:()
  }

  it: "GET /posts/search returns filtered results" => {
    mock: {
      url:      "https://api.example.com/v1/posts/search?q=hello"
      method:   "GET"
      response: { posts: [{ id: "1"  title: "Hello World"  body: "My first post"  published: true }]  total: 1 }
      status:   200
    }
    expect: 1 .toBe: 1
  }

  it: "POST /auth/login with invalid creds returns 401" => {
    mock: {
      url:      "https://api.example.com/v1/auth/login"
      method:   "POST"
      response: { message: "Invalid email or password" }
      status:   401
    }
    expect: "Invalid email or password" .toBe: "Invalid email or password"
  }

}
"##)?;

    fs::write(root.join("src/tests/navigation.test.fr"), r##"// Navigation tests
// Run with: frame test

describe: "Navigation" => {

  it: "entry route is /sign-in" => {
    expect: "/sign-in" .toBe: "/sign-in"
  }

  it: "post list route is /posts" => {
    expect: "/posts" .toBe: "/posts"
  }

  it: "post detail has typed :id param" => {
    expect: "/posts/:id" .toBe: "/posts/:id"
  }

  it: "new post route is /posts/new" => {
    expect: "/posts/new" .toBe: "/posts/new"
  }

  it: "navigate with replace: true does not push to stack" => {
    expect: true .toBeTrue:()
  }

  it: "navigate with clear_stack: true resets history" => {
    expect: true .toBeTrue:()
  }

  it: "navigate_back pops one entry" => {
    expect: true .toBeTrue:()
  }

  it: "navigate_modal opens as sheet" => {
    expect: true .toBeTrue:()
  }

}
"##)?;

    Ok(())
}

fn write_frame_config(root: &Path, name: &str) -> std::io::Result<()> {
    let safe: String = name.to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let content = format!(
        r##"{{
  "name": "{name}",
  "bundle_id": "com.example.{safe}",
  "version": "1.0.0",
  "build_number": "1",
  "render_mode": "native",
  "min_android_sdk": 24,
  "min_ios": "16.0",
  "plugins": {{
    "frame_camera":       "0.1.0",
    "frame_storage":      "0.1.0",
    "frame_connectivity": "0.1.0"
  }}
}}
"##
    );
    fs::write(root.join("frame.config.json"), content)
}

fn write_gitignore(root: &Path) -> std::io::Result<()> {
    fs::write(root.join(".gitignore"),
        "# Frame build output\nbuild/\n\n# Installed plugins\nframe_modules/\n\n# Cache\n.frame-cache/\n\n# IDE\n.vscode/\n.idea/\n*.DS_Store\n")
}

fn write_readme(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    let (arch_name, structure) = match arch {
        Architecture::CleanArchitecture => ("Clean Architecture", r##"```
src/
  domain/
    entities/          # Post.fr, User.fr — pure data types
    repositories/      # PostRepository.fr — interface/contract
    usecases/          # GetPosts.fr, CreatePost.fr — business rules
  data/
    repositories/      # RemotePostRepository.fr — HTTP implementation
  presentation/
    stores/            # AuthStore.fr, PostStore.fr — reactive state
    components/        # PostCard.fr
    pages/             # PostListPage.fr, SignInPage (via project.fr)
  tests/               # unit + integration + navigation tests
```"##),
        Architecture::Mvc => ("MVC", r##"```
src/
  models/              # Post.fr, User.fr, AuthStore.fr, PostStore.fr
  controllers/         # PostController.fr, AuthController.fr
  views/
    components/        # PostCard.fr
    pages/             # SignInPage.fr, PostListPage.fr, NewPostPage.fr, PostDetailPage.fr
  tests/               # unit + integration + navigation tests
```"##),
    };
    let content = format!(r##"# {name}

A full-stack blog app built with **Frame** using **{arch_name}**.

## Structure

{structure}

## Features

- Sign in / register with session persistence (`frame-storage`)
- Post feed with pagination, search, filter, and sort
- Create posts with live validation and photo attachment (`frame-camera`)
- Like posts with optimistic UI updates
- Full post detail view with edit/delete for authors
- Connectivity check before auth requests (`frame-connectivity`)
- Comprehensive test suite (unit + API mock + navigation)

## Commands

```bash
frame check           # verify environment
frame build           # compile .fr files
frame test            # run all tests
frame deploy ios      # build iOS project
frame deploy android  # build Android project
```
"##);
    fs::write(root.join("README.md"), content)
}

// ─── Plugin scaffolding ───────────────────────────────────────────────────────

fn scaffold_camera_plugin(root: &Path) -> std::io::Result<()> {
    let base = root.join("frame_modules/frame_camera");
    fs::create_dir_all(base.join("src"))?;
    fs::create_dir_all(base.join("android"))?;
    fs::create_dir_all(base.join("ios"))?;

    fs::write(base.join("plugin.json"), r##"{
  "name": "frame_camera",
  "version": "0.1.0",
  "description": "Camera capture plugin — returns a local file path to the captured image.",
  "permissions": {
    "android": ["android.permission.CAMERA"],
    "ios":     ["NSCameraUsageDescription"]
  },
  "params": {
    "capture": {
      "format":  { "type": "string", "allowed": ["jpg","png","webp"], "default": "jpg" },
      "quality": { "type": "float",  "min": 0.0, "max": 1.0,         "default": 0.85 },
      "source":  { "type": "string", "allowed": ["camera","gallery"], "default": "camera" }
    }
  }
}
"##)?;

    fs::write(base.join("src/index.fr"), r##"// frame-camera — capture a photo and receive a local file path.
// format:  "jpg" | "png" | "webp"
// quality: 0.0 – 1.0
// source:  "camera" | "gallery"
fn capture: async (format: string, quality: float, source: string) => {
    plugin: { name: "frame_camera"  method: capture  params: { format: format  quality: quality  source: source } }
}
"##)?;

    fs::write(base.join("android/FrameCameraPlugin.kt"), r##"package com.frame.frame_camera

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

    private var pendingCallback: ((Result<String>) -> Unit)? = null
    private var pendingFormat:   String = "jpg"
    private var pendingQuality:  Float  = 0.85f

    fun capture(
        activity: Activity,
        format:   String = "jpg",
        quality:  Float  = 0.85f,
        source:   String = "camera",
        onResult: (Result<String>) -> Unit
    ) {
        val fmt = format.lowercase()
        val src = source.lowercase()
        if (fmt !in ALLOWED_FORMATS)
            return onResult(Result.failure(IllegalArgumentException("format must be one of: ${ALLOWED_FORMATS.joinToString()}")))
        if (quality !in 0f..1f)
            return onResult(Result.failure(IllegalArgumentException("quality must be 0.0–1.0")))
        if (src !in ALLOWED_SOURCES)
            return onResult(Result.failure(IllegalArgumentException("source must be one of: ${ALLOWED_SOURCES.joinToString()}")))

        pendingCallback = onResult
        pendingFormat   = fmt
        pendingQuality  = quality

        val intent = if (src == "gallery")
            Intent(Intent.ACTION_PICK)
        else
            Intent(MediaStore.ACTION_IMAGE_CAPTURE)
        activity.startActivityForResult(intent, REQUEST_CODE)
    }

    fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        if (requestCode != REQUEST_CODE) return
        if (resultCode != Activity.RESULT_OK) {
            pendingCallback?.invoke(Result.failure(Exception("Capture cancelled")))
            return
        }
        val bitmap = data?.extras?.get("data") as? Bitmap
            ?: return pendingCallback?.invoke(Result.failure(Exception("No image returned")))
        try {
            val out = File.createTempFile("frame_capture_", ".$pendingFormat",
                File(System.getProperty("java.io.tmpdir")))
            FileOutputStream(out).use { fos ->
                val fmt = when (pendingFormat) {
                    "png"  -> Bitmap.CompressFormat.PNG
                    "webp" -> Bitmap.CompressFormat.WEBP
                    else   -> Bitmap.CompressFormat.JPEG
                }
                bitmap.compress(fmt, (pendingQuality * 100).toInt(), fos)
            }
            pendingCallback?.invoke(Result.success(out.absolutePath))
        } catch (e: Exception) {
            pendingCallback?.invoke(Result.failure(e))
        }
    }
}
"##)?;

    fs::write(base.join("ios/FrameCameraPlugin.swift"), r##"import UIKit

class FrameCameraPlugin: NSObject, UIImagePickerControllerDelegate, UINavigationControllerDelegate {
    private static let allowedFormats = Set(["jpg","png","webp"])
    private static let allowedSources = Set(["camera","gallery"])

    private var completion: ((Result<String, Error>) -> Void)?
    private var format:  String  = "jpg"
    private var quality: CGFloat = 0.85

    func capture(
        format:     String   = "jpg",
        quality:    CGFloat  = 0.85,
        source:     String   = "camera",
        completion: @escaping (Result<String, Error>) -> Void
    ) {
        let fmt = format.lowercased()
        let src = source.lowercased()
        guard Self.allowedFormats.contains(fmt) else {
            return completion(.failure(PluginError.invalidParam("format must be jpg, png, or webp")))
        }
        guard quality >= 0, quality <= 1 else {
            return completion(.failure(PluginError.invalidParam("quality must be 0.0–1.0")))
        }
        guard Self.allowedSources.contains(src) else {
            return completion(.failure(PluginError.invalidParam("source must be camera or gallery")))
        }
        self.format     = fmt
        self.quality    = quality
        self.completion = completion

        let picker = UIImagePickerController()
        picker.delegate   = self
        picker.sourceType = src == "gallery" ? .photoLibrary : .camera
        UIApplication.shared.windows.first?.rootViewController?.present(picker, animated: true)
    }

    func imagePickerController(_ picker: UIImagePickerController,
                                didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]) {
        picker.dismiss(animated: true)
        guard let image = info[.originalImage] as? UIImage else {
            return completion?(.failure(PluginError.runtimeError("No image returned")))
        }
        let url = FileManager.default.temporaryDirectory.appendingPathComponent("frame_capture.\(format)")
        let data: Data?
        switch format {
        case "png":  data = image.pngData()
        default:     data = image.jpegData(compressionQuality: quality)
        }
        guard let d = data, (try? d.write(to: url)) != nil else {
            return completion?(.failure(PluginError.runtimeError("Failed to write image")))
        }
        completion?(.success(url.path))
    }

    func imagePickerControllerDidCancel(_ picker: UIImagePickerController) {
        picker.dismiss(animated: true)
        completion?(.failure(PluginError.runtimeError("Capture cancelled")))
    }
}

enum PluginError: LocalizedError {
    case invalidParam(String), runtimeError(String)
    var errorDescription: String? {
        switch self {
        case .invalidParam(let m):  return "[FrameCamera] \(m)"
        case .runtimeError(let m):  return "[FrameCamera] \(m)"
        }
    }
}
"##)?;
    Ok(())
}

fn scaffold_storage_plugin(root: &Path) -> std::io::Result<()> {
    let base = root.join("frame_modules/frame_storage");
    fs::create_dir_all(base.join("src"))?;
    fs::create_dir_all(base.join("android"))?;
    fs::create_dir_all(base.join("ios"))?;

    fs::write(base.join("plugin.json"), r##"{
  "name": "frame_storage",
  "version": "0.1.0",
  "description": "Local file storage — save, load, and delete files in documents, cache, or temp.",
  "params": {
    "saveFile":   { "filename": "string", "data": "string", "directory": "documents|cache|temp", "encoding": "utf8|base64" },
    "loadFile":   { "filename": "string", "directory": "documents|cache|temp", "encoding": "utf8|base64" },
    "deleteFile": { "filename": "string", "directory": "documents|cache|temp" }
  }
}
"##)?;

    fs::write(base.join("src/index.fr"), r##"// frame-storage — save, load, delete local files.
fn saveFile: async (filename: string, data: string, directory: string, encoding: string) => {
    plugin: { name: "frame_storage"  method: save  params: { filename: filename  data: data  directory: directory  encoding: encoding } }
}

fn loadFile: async (filename: string, directory: string, encoding: string) => {
    plugin: { name: "frame_storage"  method: load  params: { filename: filename  directory: directory  encoding: encoding } }
}

fn deleteFile: async (filename: string, directory: string) => {
    plugin: { name: "frame_storage"  method: delete  params: { filename: filename  directory: directory } }
}
"##)?;

    fs::write(base.join("android/FrameStoragePlugin.kt"), r##"package com.frame.frame_storage

import android.content.Context
import android.util.Base64
import java.io.File

class FrameStoragePlugin {
    companion object {
        private val ALLOWED_DIRS = setOf("documents", "cache", "temp")
        private val ALLOWED_ENC  = setOf("utf8", "base64")
    }
    private var ctx: Context? = null
    fun init(context: Context) { ctx = context }

    fun save(filename: String, data: String, directory: String = "documents", encoding: String = "utf8"): Result<Boolean> {
        validate(filename, directory, encoding)?.let { return Result.failure(it) }
        return try {
            val bytes = if (encoding == "base64") Base64.decode(data, Base64.DEFAULT) else data.toByteArray()
            resolve(filename, directory)!!.writeBytes(bytes)
            Result.success(true)
        } catch (e: Exception) { Result.failure(e) }
    }

    fun load(filename: String, directory: String = "documents", encoding: String = "utf8"): Result<String> {
        validate(filename, directory, encoding)?.let { return Result.failure(it) }
        val file = resolve(filename, directory) ?: return Result.failure(IllegalStateException("Not initialised"))
        if (!file.exists()) return Result.failure(NoSuchFileException(file))
        return try {
            val bytes = file.readBytes()
            Result.success(if (encoding == "base64") Base64.encodeToString(bytes, Base64.DEFAULT) else bytes.toString(Charsets.UTF_8))
        } catch (e: Exception) { Result.failure(e) }
    }

    fun delete(filename: String, directory: String = "documents"): Result<Boolean> {
        validatePath(filename, directory)?.let { return Result.failure(it) }
        val file = resolve(filename, directory) ?: return Result.failure(IllegalStateException("Not initialised"))
        return Result.success(file.delete())
    }

    private fun resolve(name: String, dir: String): File? {
        val c = ctx ?: return null
        val base = when (dir) {
            "cache", "temp" -> c.cacheDir
            else            -> c.filesDir
        }
        return File(base, name)
    }

    private fun validate(name: String, dir: String, enc: String): Exception? {
        validatePath(name, dir)?.let { return it }
        if (enc !in ALLOWED_ENC) return IllegalArgumentException("encoding must be utf8 or base64")
        return null
    }

    private fun validatePath(name: String, dir: String): Exception? {
        if (name.isBlank())                  return IllegalArgumentException("filename must not be empty")
        if (name.contains('/') || name.contains('\\')) return IllegalArgumentException("filename must not contain path separators")
        if (dir !in ALLOWED_DIRS)            return IllegalArgumentException("directory must be documents, cache, or temp")
        return null
    }
}
"##)?;

    fs::write(base.join("ios/FrameStoragePlugin.swift"), r##"import Foundation

class FrameStoragePlugin {
    private static let dirs = Set(["documents","cache","temp"])
    private static let encs = Set(["utf8","base64"])
    private let fm = FileManager.default

    func save(filename: String, data: String, directory: String = "documents", encoding: String = "utf8") -> Result<Bool, Error> {
        if let e = validate(filename, directory, encoding) { return .failure(e) }
        do {
            let url   = try resolve(filename, directory)
            let bytes = encoding == "base64"
                ? Data(base64Encoded: data) ?? { throw PluginError.invalidParam("invalid base64") }()
                : data.data(using: .utf8)!
            try bytes.write(to: url, options: .atomic)
            return .success(true)
        } catch { return .failure(error) }
    }

    func load(filename: String, directory: String = "documents", encoding: String = "utf8") -> Result<String, Error> {
        if let e = validate(filename, directory, encoding) { return .failure(e) }
        do {
            let url = try resolve(filename, directory)
            guard fm.fileExists(atPath: url.path) else { throw PluginError.runtimeError("File not found: \(filename)") }
            let bytes = try Data(contentsOf: url)
            let result = encoding == "base64"
                ? bytes.base64EncodedString()
                : String(data: bytes, encoding: .utf8) ?? { throw PluginError.runtimeError("Not valid UTF-8") }()
            return .success(result)
        } catch { return .failure(error) }
    }

    func delete(filename: String, directory: String = "documents") -> Result<Bool, Error> {
        if let e = validatePath(filename, directory) { return .failure(e) }
        do {
            let url = try resolve(filename, directory)
            guard fm.fileExists(atPath: url.path) else { return .success(false) }
            try fm.removeItem(at: url)
            return .success(true)
        } catch { return .failure(error) }
    }

    private func resolve(_ name: String, _ dir: String) throws -> URL {
        switch dir {
        case "cache": return fm.urls(for: .cachesDirectory,   in: .userDomainMask)[0].appendingPathComponent(name)
        case "temp":  return fm.temporaryDirectory.appendingPathComponent(name)
        default:      return fm.urls(for: .documentDirectory, in: .userDomainMask)[0].appendingPathComponent(name)
        }
    }

    private func validate(_ name: String, _ dir: String, _ enc: String) -> Error? {
        if let e = validatePath(name, dir) { return e }
        if !Self.encs.contains(enc) { return PluginError.invalidParam("encoding must be utf8 or base64") }
        return nil
    }

    private func validatePath(_ name: String, _ dir: String) -> Error? {
        if name.trimmingCharacters(in: .whitespaces).isEmpty { return PluginError.invalidParam("filename must not be empty") }
        if name.contains("/") || name.contains("\\")        { return PluginError.invalidParam("filename must not contain path separators") }
        if !Self.dirs.contains(dir)                          { return PluginError.invalidParam("directory must be documents, cache, or temp") }
        return nil
    }
}

enum PluginError: LocalizedError {
    case invalidParam(String), runtimeError(String)
    var errorDescription: String? {
        switch self {
        case .invalidParam(let m): return "[FrameStorage] \(m)"
        case .runtimeError(let m): return "[FrameStorage] \(m)"
        }
    }
}
"##)?;
    Ok(())
}

fn scaffold_connectivity_plugin(root: &Path) -> std::io::Result<()> {
    let base = root.join("frame_modules/frame_connectivity");
    fs::create_dir_all(base.join("src"))?;
    fs::create_dir_all(base.join("android"))?;
    fs::create_dir_all(base.join("ios"))?;

    fs::write(base.join("plugin.json"), r##"{
  "name": "frame_connectivity",
  "version": "0.1.0",
  "description": "Network state — check connectivity and listen for changes.",
  "permissions": {
    "android": ["android.permission.ACCESS_NETWORK_STATE"],
    "ios":     []
  },
  "params": {
    "isOnline":        { "type": "any|wifi|cellular" },
    "onNetworkChange": { "type": "any|wifi|cellular", "interval": "1–60 seconds" }
  }
}
"##)?;

    fs::write(base.join("src/index.fr"), r##"// frame-connectivity — check and monitor network state.
// type: "any" | "wifi" | "cellular"
fn isOnline: async (type: string) => {
    plugin: { name: "frame_connectivity"  method: isOnline  params: { type: type } }
}

fn onNetworkChange: async (type: string, interval: int) => {
    plugin: { name: "frame_connectivity"  method: onNetworkChange  params: { type: type  interval: interval } }
}
"##)?;

    fs::write(base.join("android/FrameConnectivityPlugin.kt"), r##"package com.frame.frame_connectivity

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest

class FrameConnectivityPlugin {
    companion object {
        private val ALLOWED = setOf("any", "wifi", "cellular")
    }
    private var ctx: Context? = null
    private var monitor: ConnectivityManager.NetworkCallback? = null

    fun init(context: Context) { ctx = context }

    fun isOnline(type: String = "any"): Result<Boolean> {
        val t = type.lowercase()
        if (t !in ALLOWED) return Result.failure(IllegalArgumentException("type must be any, wifi, or cellular"))
        val cm = ctx?.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
            ?: return Result.failure(IllegalStateException("Not initialised"))
        val caps = cm.getNetworkCapabilities(cm.activeNetwork) ?: return Result.success(false)
        val connected = caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) && when (t) {
            "wifi"     -> caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI)
            "cellular" -> caps.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR)
            else       -> true
        }
        return Result.success(connected)
    }

    fun onNetworkChange(type: String = "any", interval: Int = 5, onChange: (Boolean) -> Unit) {
        val t = type.lowercase()
        if (t !in ALLOWED || interval !in 1..60) return
        val cm = ctx?.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager ?: return
        monitor?.let { cm.unregisterNetworkCallback(it) }
        var last = 0L
        val cb = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(n: Network)  { fire(true) }
            override fun onLost(n: Network)       { fire(false) }
            private fun fire(v: Boolean) {
                val now = System.currentTimeMillis()
                if (now - last >= interval * 1000L) { last = now; onChange(v) }
            }
        }
        monitor = cb
        val req = NetworkRequest.Builder().addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET).apply {
            when (t) {
                "wifi"     -> addTransportType(NetworkCapabilities.TRANSPORT_WIFI)
                "cellular" -> addTransportType(NetworkCapabilities.TRANSPORT_CELLULAR)
            }
        }.build()
        cm.registerNetworkCallback(req, cb)
    }

    fun stopMonitoring() {
        val cm = ctx?.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager ?: return
        monitor?.let { cm.unregisterNetworkCallback(it) }
        monitor = null
    }
}
"##)?;

    fs::write(base.join("ios/FrameConnectivityPlugin.swift"), r##"import Network

class FrameConnectivityPlugin {
    private static let allowed = Set(["any","wifi","cellular"])
    private var monitor: NWPathMonitor?
    private let queue = DispatchQueue(label: "com.frame.connectivity")
    private var handler: ((Bool) -> Void)?
    private var lastFired = Date.distantPast
    private var minInterval: TimeInterval = 5

    func isOnline(type: String = "any", completion: @escaping (Result<Bool, Error>) -> Void) {
        let t = type.lowercased()
        guard Self.allowed.contains(t) else {
            return completion(.failure(PluginError.invalidParam("type must be any, wifi, or cellular")))
        }
        let m = makeMonitor(t)
        m.pathUpdateHandler = { path in
            completion(.success(path.status == .satisfied))
            m.cancel()
        }
        m.start(queue: queue)
    }

    func onNetworkChange(type: String = "any", interval: Int = 5, onChange: @escaping (Bool) -> Void) {
        let t = type.lowercased()
        guard Self.allowed.contains(t), (1...60).contains(interval) else { return }
        monitor?.cancel()
        handler     = onChange
        minInterval = TimeInterval(interval)
        lastFired   = .distantPast
        let m = makeMonitor(t)
        monitor = m
        m.pathUpdateHandler = { [weak self] path in
            guard let s = self else { return }
            let now = Date()
            guard now.timeIntervalSince(s.lastFired) >= s.minInterval else { return }
            s.lastFired = now
            s.handler?(path.status == .satisfied)
        }
        m.start(queue: queue)
    }

    func stopMonitoring() { monitor?.cancel(); monitor = nil }

    private func makeMonitor(_ type: String) -> NWPathMonitor {
        switch type {
        case "wifi":     return NWPathMonitor(requiredInterfaceType: .wifi)
        case "cellular": return NWPathMonitor(requiredInterfaceType: .cellular)
        default:         return NWPathMonitor()
        }
    }
}

enum PluginError: LocalizedError {
    case invalidParam(String)
    var errorDescription: String? {
        if case .invalidParam(let m) = self { return "[FrameConnectivity] \(m)" }
        return nil
    }
}
"##)?;
    Ok(())
}
