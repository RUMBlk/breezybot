activities = Activities
participants = Participants

activity-stat-table = Right now members of {$guild} are participating in
    ```{$table}```
activity-stat-empty = Currently there are no members in this server that participate in any activity.

activity-leaderboard = leaderboard
    .description = Activity leaderboard. Calculated from posted messages.

activity-leaderboard-empty = The leaderboard is empty, please try later.

index = No.
members = Members
points = Points
candidates = Candidates
share = Share
role = Role
scheduled_date = Scheduled Date
schedule = Schedule

activity-leaderboard-table = {$guild} leaderboard
    ```{$table}``````Server value: {$server_value}```
    .server-value-err = Failed to calculate server score.

elections-announcement = ## Election results
    .role = ### Role: <@&{$role}>
    .promoted = Promoted: 
    .demoted = Demoted: 
    .failed = (Failed)
    .scheduled_for = Scheduled next {$role} elections for {$date}

activity = activity
    .description = Activity analytics.

activity-stat = stat
    .description = List of activities members of the server do right now.

elections-leaderboard = leaderboard
    .description = The leaderboard of candidates and their shares in the elections of the specified role
elections-leaderboard-table = {$role} elections leaderboard
    ```{$table}```
elections-leaderboard-table-empty = There's no {$role} candidates at the moment!

elections-announcements = announcements
    .description = Set channel for election announcements, by default it's the server's system channel.

elections-force = force
    .description = Force elections for the specified role.

elections-claims_add = claims_add
    .description = Take participation in the election for the specified role.

elections-claims_remove = claims_remove
    .description = Leave elections for the specified role.

elections-claims_kick = claims_kick
    .description = Kicks the specified member from the provided role elections.

elections-claims_ban = claims_ban
    .description = Bans the specified members from the provided role elections.

elections-claims_unban = claims_unban
    .description = Unbans the specified members from the provided role elections.

elections-roles_list = roles_list
    .description = List of opened elections in the server.

elections-roles_add = roles_add
    .description = Opens elections for the specified role.

elections-roles_edit = roles_edit
    .description = Edits parameters of the specified election.

elections-roles_delete = roles_delete
    .description = Cancels elections for the specified role.

elections-votes_list = votes_list
    .description = The table of candidates you vote for in elections of the specified role.

elections-votes_cast = votes_cast
    .description = Casts a vote for the specified member in elecetions of the provided role.

elections-votes_remove = votes_remove
    .description = Removes your vote casted for the specified member in the provided role's elections.

elections-channel-not-updated = The channel you provided matches the previous set.
elections-channel = Noted! From now on I'll post all election announcements in {$channel}!
elections-channel-system = Noted! From now on I'll post all election announcements in the server's system channel!

cmd-not-in-guild = This command can be executed only in a guild!
database-unreachable = The database is unreachable, please try later.
database-oops = Failed to execute the operation. Please verify the correctness of provided arguments.
unknown-highest-role = Couldn't determine your highest role.
insufficient_role_position = Your highest's role position is either lower than or equal with {$role}.
role-unavailable = Couldn't retrieve the role data. 
elections-not-found = **{$role}** is not electable!
elections-not-scheduled = The elections for {$role} are not scheduled. Set "force" argument to "True" to host them.
elections-scheduled-for-later = The scheduled date for the {$role} elections is {$scheduled_date}. Set "force" argument to "True" to host them.
elections-forced = Successfully hosted the {$role} elections.
elections-candidate-exists = You already claim {$role}!
elections-claim-added = {$user} becomes a candidate of {$role} elections!
elections-claim-not-found = You are not claiming **{$role}**!
elections-claim-removed = **{$user}** draws back their claim for the **{$role}** elections!
elections-claim-banned = Banned **{$user}** from participating in **{$role}** elections until <t:{$banned_until}:f>
elections-claim-already-banned = **{$user} is already banned in the **{$role}** elections.
elections-claim-unbanned = Successfully unbanned **{$user}** in the **{$role}** elections.
elections-claim-not-banned = **{$user}** is not banned in the **{$role}** elections!
elections-roles-list = Opened elections in the server:\n```{$table}```
elections-roles-list-empty = There's no opened elections in the server at the moment!
elections-roles-add = Elections for **{$role}** have started!
elections-roles-exists = **{$role}** is already electable!
elections-roles-edit = ### Set new parameters for **{$role}** elections:
    Number of available positions: {$number_of_positions}
    Schedule: {$schedule}
elections-roles-delete = Elections for **{$role}** have closed!
elections-candidate-not-found = {$candidate} doesn't claim {$role}.
elections-votes-list = The table of {$role} candidates you vote for:\n```{$table}```
elections-votes-empty = You don't vote for anyone who claims {$role} role.
elections-vote-casted = A vote for {$candidate} in {$role} elecetions has been casted!
elections-vote-exists = You already support {$candidate} in the {$role} elections!
elections-vote-remove = A vote for {$candidate} in the {$role} elections has been removed!
elections-vote-not-found = You don't vote for {$candidate} in the {$role} elections!