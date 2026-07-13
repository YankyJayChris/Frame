# profile

A full-stack blog app built with **Frame** using **Clean Architecture**.

## Structure

```
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
