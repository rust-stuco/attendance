A robust rewrite of our previous mailing system.

Previously a Google Apps Script that twiddled Google Sheets cells, now a full-fledged system with more safeguards and automates even more steps. Written in Rust, of course.

# Setup

## Mailer Setup

In `config.toml`, you can set the sender, CC'd contacts, and email body.

As of now, sender must use Gmail SMTP! Recipients and CC'd can be any.

1. Copy `.env.example` to a new `.env` file
2. Enable 2FA for your sender's gmail account
3. Generate a password from https://security.google.com/settings/security/apppasswords
4. Paste the password into `.env` file

## Attendance Tracker Setup

Next, create your `roster.json` and `weekly_data.json` files and update their paths in `config.toml` accordingly.

See `examples` for reference of how these json files are structured.

# Usage

1. Run `bump-week` at the beginning of each week.
2. Throughout the week, update the roster and excused absences with `add-student` `remove-student`, `mark-excused` respectively.
3. Mark attendees with `mark-attended` or `bulk-mark-attended <file_path>`.
4. When it comes time to email absentees, pull the trigger with `email-unexcused`. A final confirmation prompt will appear with a preview of the recipients, as an extra layer of protection against accidental emails.

## Bulk Mark Attendance

`bulk-mark-attended` expects a file with Andrew IDs separated by line. It also accepts email addresses, taking the part before the `@` as the Andrew ID.

See `examples/attendees.txt` for reference.

## Miscellaneous Commands

You can list all available commands by running the tool with blank or invalid commands. Some other commands you may want to use:
* `aggregate-stats` calculates the number of unexcused absences for each student
* `flag-at-risk` filters for students with unexcused absences above the threshold 2
* `show-week` for summary of weekly data
* `set-week <week_number>` to navigate to an older week

# TODO


[] Add option to cancel emails within time window