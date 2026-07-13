# blog-app

A full-stack blog app built with **Frame** using **MVC**.

## Structure

```
src/
  models/              # Post.fr, User.fr, AuthStore.fr, PostStore.fr
  controllers/         # PostController.fr, AuthController.fr
  views/
    components/        # PostCard.fr
    pages/             # SignInPage.fr, PostListPage.fr, NewPostPage.fr, PostDetailPage.fr
  tests/               # unit + integration + navigation tests
```

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
