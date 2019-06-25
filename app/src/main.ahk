Get() {
	Input, Got, L1
	return Got
}

DoBackspace() {
	Send, BackSpace
}

DoWrite(*String) {
	SendRaw, String
}

GetAddr := RegisterCallback("Get" [, Options = "Fast"])
BackspaceAddr := RegisterCallback("DoBackspace" [, Options = "Fast"])
WriteAddr := RegisterCallback("DoWrite" [, Options = "Fast"])

DllCall("../../target/release/eggs_dee.dll/run", [, Uint, 8, Ptr, GetAddr, Ptr, BackspaceAddr, Ptr, WriteAddr])