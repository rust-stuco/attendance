A robust rewrite of our previous mailing system.

Previously a Google Apps Script that twiddled Google Sheets cells, now a full-fledged system with more safeguards and automates even more steps. Written in Rust, of course.

# Setup

## Mailer Setup

You can set the sender and CC'd email addresses in `config.toml`.

As of now, sender must use Gmail SMTP! Recipients and CC'd can be any.

1. Copy `.env.example` to a new `.env` file
2. Enable 2FA for your sender's gmail account
3. Generate a password from https://security.google.com/settings/security/apppasswords
4. Paste the password into `.env` file

## Attendance Tracker Setup

Next, create your `roster.json` and `weekly_data.json` files and update their paths in `config.toml` accordingly.

`roster.json.example` and `weekly_data.json.example` have been included for reference.

# Usage

1. Run `bump-week` and `reset-week` at the beginning of each week. These steps are important! `bump-week` increments the current week, and `reset-week` creates an empty entry for the current week number.
2. Update the roster and excused absences with `add-student` `remove-student`, `mark-excused`, `mark-attended` respectively.
3. Before you email, you can view the unexcused absences with `list-unexcused`.
4. Then, when it comes time to email absentees, pull the trigger with `email-unexcused`. A final confirmation prompt will appear, as an extra layer of protection against accidental emails.


# TODO

[x] Get mailing functionality working

[x] Manage roster add/removes

[x] Manage excusals

[] Batch mark attended from file

[x] Add warning and confirmation before emails are sent

[] Add option to cancel emails within window