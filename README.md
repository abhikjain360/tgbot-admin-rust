# Telegram Bot in Rust for Heroku

A simple Telegram bot, with admin-bot-like functionalities. Built upon [Teloxide](https://github.com/teloxide/teloxide) and uses this [buildpack](https://github.com/emk/heroku-buildpack-rust). I have done nothing more than combine these 2 together.

## banning certain words in group

All such words need to kept in `banned_words` file.
> NOTE: All words are taken as case-insensitive, and all text is also filtered by first making them case-insensitive. Only single words are supported for now.

## Admin access to bot

The telegram id (the numeric one, not the username) of all admins should be added to `admins` file.

## Admin powers

- Add new words to ban using `/add <word>` command.
- Add new admins using `/addadmin <admin-id>` command. Only numeric id will work.

## General usage

| Command     | Action                                |
|-------------|---------------------------------------|
| `/help`     | display help                          |
| `/list`     | list all banned words                 |
| `/add`      | add new banned word (admin exclusive) |
| `/addadmin` | add new admin (admin exclusive)       |

## Deploying on Heroku

- Create an Heroku app.

- Add heroku to your repo.
```
heroku git:remote -a <your-app-name>
```

- Use this [buildpack](https://github.com/emk/heroku-buildpack-rust).
```
heroku buildpacks:set emk/rust
```

- Create `Procfile` by copying `ProcfileExample`
```
cp ProcfileExample Procfile
```

- Add your [Telegram Bot Token](https://core.telegram.org/bots#3-how-do-i-create-a-bot) to `Procfile`.

- test locally.
  * release mode
    ```
    cargo build --release
    sh -c  "$(cat Procfile | cut -d ':' -f2,3)"
    ```
  * debug mode
    ```
    TELOXIDE_TOKEN=<enter your token here> cargo run
    ```

- Stage, commit and push.
```
git add .
git commit -m "<commit-message-here>"
git push heroku master
```
