[CmdletBinding()]
param(
    [string]$CardPath = (Join-Path $PSScriptRoot 'agentcard.json'),
    [string]$SchemaPath = (Join-Path $PSScriptRoot 'agentcard.schema.json'),
    [Parameter(Mandatory)][string]$ExpectedRepository
)
$ErrorActionPreference = 'Stop'
$json = Get-Content -LiteralPath $CardPath -Raw
if (-not (Test-Json -Json $json -SchemaFile $SchemaPath)) { throw 'Invalid agent card.' }
$card = $json | ConvertFrom-Json -AsHashtable
if ($card.repository -cne $ExpectedRepository -or $card.displayName -cne $ExpectedRepository) {
    throw 'The card does not belong to the expected repository.'
}
$fields = [ordered]@{
    code = @('defaultBranch', 'commit', 'branches', 'tags', 'language', 'lastCommitDate', 'releaseVersion', 'releaseCount')
    issues = @('enabled', 'openCount', 'countsObservedAt')
    'pull-requests' = @('totalCount', 'openCount', 'closedCount', 'mergedCount', 'historyObservedAt', 'collectionStatus')
    agents = @('definitions', 'skills', 'guidance', 'activeSessions')
    actions = @('defaultCommitRollup', 'checksObservedAt', 'workflowFiles', 'workflowRunCount', 'loadedCount', 'hasMore', 'checksHaveMore')
    projects = @('enabled')
    wiki = @('enabled', 'pageCount')
    security = @('assessment')
    insights = @('stars', 'forks', 'watchers', 'lastPush')
    settings = @('visibility', 'archived', 'disabled', 'template', 'defaultBranch', 'allowForking', 'issuesEnabled', 'wikiEnabled', 'projectsEnabled', 'discussionsEnabled')
}
foreach ($key in $fields.Keys) {
    $tab = $card.tabs[$key]
    if ($tab.value.repository -cne $ExpectedRepository) { throw "Repository mismatch in $key." }
    if (@($tab.value.data.Keys).Count -ne $fields[$key].Count) { throw "Unexpected fields in $key." }
    foreach ($field in $tab.value.data.Keys) {
        if ($field -cnotin $fields[$key]) { throw "Nonportable field: $key.$field." }
    }
    if (-not (Test-Json -Json ($tab.value | ConvertTo-Json -Depth 50) -Schema ($tab.schema | ConvertTo-Json -Depth 50))) {
        throw "Invalid tab value: $key."
    }
}
if ($card.tabs.security.value.data.assessment -cne 'not-exported') {
    throw 'Private security assessments must not be published in a portable card.'
}
[ordered]@{
    agent = 'darbot'
    repository = $ExpectedRepository
    valid = $true
    tabs = $fields.Count
    observedAt = $card.observedAt
    profile = 'portable-repository-metadata'
} | ConvertTo-Json -Compress
