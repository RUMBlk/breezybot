activity = activity
    .description = Activity analytics

activity-stat = stat
    .description = List of activities members of the server do right now
activities = Activities
participants = Participants

activity-stat-participants = Right now members of {$guild} are participating in
    ```{$table}```
activity-stat-no-participants = Currently there are no members in this server that participate in any activity.

activity-leaderboard = leaderboard
    .description = Activity leaderboard. Calculated from posted messages.

activity-leaderboard-empty = The leaderboard is empty, please try later.
index = No.
members = Members
points = Points
activity-leaderboard-table = {$guild} leaderboard
    ```{$table}``````Server value: {$server_value}```
    .server-value-err = Failed to calculate server score.

elections-announcement = ## Election results
    .role = ### Role: <@&{$role}>
    .promoted = Promoted: 
    .demoted = Demoted: 
    .scheduled_for = Scheduled next {$role} elections for {$date}

cmd-not-in-guild = This command can be executed only in a guild!
database-unreachable = The database is unreachable, please try later.
database-oops = Failed to execute the operation. Please verify the correctness of provided arguments.

elections-not-found = **{$role}** is not electable!