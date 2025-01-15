package com.example.zenohapp.ui.examples

import io.zenoh.Zenoh
import io.zenoh.config.WhatAmI
import io.zenoh.scouting.Hello
import io.zenoh.scouting.Scout
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch

class ZScoutFragment : ZExampleFragment() {

    private var scout: Scout<Channel<Hello>>? = null

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        scout = Zenoh.scout(Channel(), whatAmI = setOf(WhatAmI.Peer, WhatAmI.Router)).getOrThrow()
        writeToConsole("Started ZScout example...")
        GlobalScope.launch {
            for (hello in scout!!.receiver) {
               writeToConsole(hello.toString())
            }
        }
    }

    override fun stopExample() {
        writeToConsole("Stopped ZScout example.")
        scout?.close()
    }

    override fun onDestroyView() {
        super.onDestroyView()
        stopExample()
    }
}