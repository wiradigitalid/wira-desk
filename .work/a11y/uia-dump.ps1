param([int]$ProcessId)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$ErrorActionPreference = 'Stop'

$root = [System.Windows.Automation.AutomationElement]::RootElement
$cond = New-Object System.Windows.Automation.PropertyCondition(
    [System.Windows.Automation.AutomationElement]::ProcessIdProperty, $ProcessId)
$win = $root.FindFirst([System.Windows.Automation.TreeScope]::Children, $cond)
if (-not $win) { Write-Output "NO WINDOW for pid $ProcessId"; exit 1 }

function Dump($el, $depth) {
    $pad = ' ' * ($depth * 2)
    $name = $el.Current.Name
    $ct   = $el.Current.ControlType.ProgrammaticName -replace '^ControlType\.',''
    $kb   = $el.Current.AccessKey
    $help = $el.Current.HelpText
    $focusable = $el.Current.IsKeyboardFocusable
    $focused   = $el.Current.HasKeyboardFocus
    $line = "$pad- [$ct] name='$name'"
    if ($focusable) { $line += " focusable" }
    if ($focused)   { $line += " **FOCUSED**" }

    # Toggle state (checkboxes / switches)
    $tp = $null
    if ($el.TryGetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern, [ref]$tp)) {
        $line += " toggle=$($tp.Current.ToggleState)"
    }
    # Value (text fields, shortcut captures)
    $vp = $null
    if ($el.TryGetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern, [ref]$vp)) {
        $line += " value='$($vp.Current.Value)' readonly=$($vp.Current.IsReadOnly)"
    }
    # Range (stack width percent)
    $rp = $null
    if ($el.TryGetCurrentPattern([System.Windows.Automation.RangeValuePattern]::Pattern, [ref]$rp)) {
        $line += " range=$($rp.Current.Value)[$($rp.Current.Minimum)..$($rp.Current.Maximum)]"
    }
    # Selection (pane tabs)
    $sp = $null
    if ($el.TryGetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern, [ref]$sp)) {
        $line += " selected=$($sp.Current.IsSelected)"
    }
    if ($kb)   { $line += " accessKey='$kb'" }
    if ($help) { $line += " help='$help'" }
    Write-Output $line

    $child = $el.FindFirst([System.Windows.Automation.TreeScope]::Children,
                           [System.Windows.Automation.Condition]::TrueCondition)
    $walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
    while ($child) {
        Dump $child ($depth + 1)
        $child = $walker.GetNextSibling($child)
    }
}
Write-Output "=== UIA tree, pid $ProcessId ==="
Dump $win 0
