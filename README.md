A robust rewrite of our previous mailing system.

Previously a Google Apps Script that twiddled Google Sheets cells, now a full-fledged system with more safeguards and automates even more steps. Written in Rust, of course.

# Setup

As of now, sender must use Gmail SMTP! Recipients and cc'd can be any.

1. Copy `.env.example` to a new `.env` file
2. Enable 2FA for your sender's gmail account
3. Generate a password from https://security.google.com/settings/security/apppasswords
4. Paste the password into `.env` file

# Usage

Update the sender and recipient info `config.toml`, then run the mailer.

# TODO

[x] Get mailing functionality working

[] Manage roster add/removes

[] Manage excusals

[] Add warning and confirmation before emails are sent

[] Add option to cancel emails within window