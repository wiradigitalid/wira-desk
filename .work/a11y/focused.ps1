param([int]$ProcessId)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$f = [System.Windows.Automation.AutomationElement]::FocusedElement
if ($f -and $f.Current.ProcessId -eq $ProcessId) {
  $ct = $f.Current.ControlType.ProgrammaticName -replace '^ControlType\.',''
  Write-Output "[$ct] '$($f.Current.Name)'"
} else { Write-Output "(focus outside target pid)" }
