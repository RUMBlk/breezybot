# Main
bot_is_ready = The bot is ready
processing = Processing...
no_desc = no_desc
indev = not done yet
denied = Sorry, this command is only for admins
bot_denied = Sorry, I do not have the "{ permission }" permission in the channel to do that!
channel_denied = Sorry, this command is only available for those who have the "{ permission }" permission in the channel.
exception = Seems like something went wrong :/\nPinging dev: <@431451036151185408>

# Permissions
permissions-manage_roles = Manage Roles
permissions-manage_webhooks = Manage Webhooks
permissions-administrator = Administrator
permissions-owner = Owner
permissions-send_messages = Send Messages

# Database
author_not_found = For some reason you're not in my database, try to post a message at first.

errors-zero_luck = Lorem Ipsum
errors-discord-cache = The cache doesn't contain required data at the moment, please try later.
errors-discord-insufficient_role_position = Your highest's role position is either lower than or equal with { role }, if it's not the case that's my bad, please ask the owner of the server to execute the command or try later.
errors-database-oops = Oops! Something went wrong with the database!
errors-database-unreachable = The database is unreachable, please try later.

elections-desc = Elections commands
elections-system = Congratulations! { elected_members } have won the { role } election!

elections-announcement-header = ## Election results
elections-announcement-role = ### Role: <@&{ role }>
elections-announcement-promoted = Promoted: 
elections-announcement-demoted = Demoted: 

elections-force-lack_permissions = { role } is higher than my highest role, I can't manage it!
elections-force-success = Successfully hosted the elections, check the server's system channel for changes.\nScheduled next elections for <t:%{ date }:D>
elections-force-election_not_found = { role } is not electable!

elections-roles-list-desc = List of opened elections in the server
elections-roles-list-table-index = No.
elections-roles-list-table-name = Role
elections-roles-list-table-scheduled_for = Scheduled for
elections-roles-list-table-hosted_each = Hosted each
elections-roles-list-success = Opened elections in the server:\n```{ table }```
elections-roles-list-empty = Currently there's no opened elections in the server!
elections-roles-add-desc = Open elections for the specified role
elections-roles-add-exists = { role } is already electable!
elections-roles-add-success = Elections for { role } started!
elections-roles-edit-desc = Edit parameters of the specified election
elections-roles-edit-success = ### Set new parameters for { role } elections:\nNumber of available positions:\n{ number_of_positions }\nHosted each: { period }
elections-roles-edit-not_found = { role } is not electable!
elections-roles-delete-desc = Cancel elections for the specified role
elections-roles-delete-success = Elections for { role } closed!
elections-roles-delete-not_found = There's no ongoing elections for { role } to cancel!

elections-claims-add-desc = Take participation in the election for the specified role
elections-claims-add-elections_not_found = { role } is not electable!
elections-claims-add-exists = You already claim { role }!
elections-claims-add-success = { user } becomes a candidate of { role } elections!
elections-claims-delete-desc = Leave { role } elections
elections-claims-delete-success = { user } draws back their candidature for the { role } elections!
elections-claims-delete-banned = You can't unclaim { role } until you're unbanned from participating in the elections!
elections-claims-delete-claim_not_found = You are not claiming { role }!
elections-claims-delete-election_not_found = { role } is not electable!
elections-claims-edit_ban-claim_not_found = Something went wrong with the database, please try again.
elections-claims-edit_ban-election_not_found = { role } is not electable!
elections-claims-ban-desc = Ban users from elections of the specified role
elections-claims-ban-success = Banned { user } from participating in { role } elections until <t:%{ banned_until }:f>

elections-votes-list-desc = The table of candidates you vote for in elections of the specified role
elections-votes-list-table-index = No.
elections-votes-list-table-candidates = Candidates
elections-votes-list-success = The table of { role } candidates you vote for:\n```{ table }```
elections-votes-list-empty = You don't vote for anyone who claims { role } role
elections-votes-add-desc = Vote for the specified member in elections of the provided role
elections-votes-add-exists = You already support { candidate } in { role } elections!
elections-votes-add-success = A vote for { candidate } in { role } has been added!
elections-votes-add-candidate_not_found = { candidate } does not claim { role } or there's no ongoing elections for { role }!
elections-votes-delete-desc = Remove your vote for the specified member in elections of the provided role
elections-votes-delete-vote_not_found = You don't vote for { candidate } in { role } elections!
elections-votes-delete-candidate_not_found = { candidate } does not claim { role } or there's no ongoing elections for { role }!
elections-votes-delete-success = A vote for { candidate } in { role } elections has been removed!
elections-votes-delete-fail = You don't support { claimer } to unsupport them!
elections-votes-delete-banned = You can't remove your claim from { role } elections until you're unbanned!
elections-leaderboard-desc = The leaderboard of candidates and their shares in the elections of the specified role
elections-leaderboard-title = { role } leaderboard
elections-leaderboard-table-index = No.
elections-leaderboard-table-candidates = Candidates
elections-leaderboard-table-share = Share
elections-leaderboard-success = { table }
elections-leaderboard-empty = There's no { role } candidates at the moment!
elections-leaderboard-election_not_found = { role } is not electable!

elections-announcements-desc = Set channel where I should post all election announcements, by default it's the server's system channel from the settings.
elections-announcements-success = Noted! From now on I'll post all election announcements in { channel }!
elections-announcements-system = Noted! From now on I'll post all election announcements in the server's system channel!

activity = activity
.description = lolz
stat = stat
    .description = List of activities the members of the server do right now
    .table = table
        .activities = Activities
        .amount = Participants
    .success = Right now members of { guild } are participating in \n```{ table }```
leaderboard = leaderboard
    .description = Leaderboard of activity of the server's members
    .empty = The leaderboard is empty, please try later.
    .table = table
        .index = No.
        .members = Members
        .points = Points
    .success = { guild } leaderboard\n```{ table }``````Server value: { server_value }``

# Message Filter
message_filter-desc = Filter configuration commands
message_filter-timezone-desc = Specify your timezone for the time filter
message_filter-timezone-timezone_limit_error = The timezone must be a number with + or - at beginning. Examples: +3; -8; +10; -12
message_filter-timezone-success = The timezone has been successfully set in the server!
