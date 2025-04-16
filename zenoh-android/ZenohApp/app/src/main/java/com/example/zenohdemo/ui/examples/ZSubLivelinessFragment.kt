package com.example.zenohdemo.ui.examples

import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.pubsub.Subscriber
import io.zenoh.sample.Sample
import io.zenoh.sample.SampleKind
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch

class ZSubLivelinessFragment : ZExampleFragment() {

    private var subscriber: Subscriber<Channel<Sample>>? = null

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        val session = viewModel.zenohSession!!
        val keyExpr = "group1/**".intoKeyExpr().getOrThrow()
        subscriber = session.liveliness().declareSubscriber(keyExpr, channel = Channel()).getOrThrow()
        writeToConsole("Declaring liveliness subscriber on '$keyExpr'...")
        GlobalScope.launch(Dispatchers.IO) {
            for (sample in subscriber!!.receiver) {
                when (sample.kind) {
                    SampleKind.PUT -> writeToConsole(">> [LivelinessSubscriber] New alive token ('${sample.keyExpr}')")
                    SampleKind.DELETE -> writeToConsole(">> [LivelinessSubscriber] Dropped token ('${sample.keyExpr}')")
                }
            }
        }

    }

    override fun stopExample() {
        subscriber?.close()
    }
}