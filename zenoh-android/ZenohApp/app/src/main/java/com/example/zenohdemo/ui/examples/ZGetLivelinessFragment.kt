package com.example.zenohdemo.ui.examples

import io.zenoh.keyexpr.intoKeyExpr
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import java.time.Duration

class ZGetLivelinessFragment : ZExampleFragment() {

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        val session = viewModel.zenohSession!!
        val keyExpr = "group1/**".intoKeyExpr().getOrThrow()

        writeToConsole("Getting liveliness on '$keyExpr'...")
        val channel =
            session.liveliness()
                .get(keyExpr, channel = Channel(), timeout = Duration.ofMillis(10000)).getOrThrow()
        GlobalScope.launch(Dispatchers.IO) {
            for (reply in channel) {
                reply.result.onSuccess {
                    writeToConsole(">> Alive token ('${it.keyExpr}')")
                }.onFailure {
                    writeToConsole(">> Received (ERROR: '${it.message}')")
                }
            }
            resetState()
        }
    }

    override fun stopExample() {}
}