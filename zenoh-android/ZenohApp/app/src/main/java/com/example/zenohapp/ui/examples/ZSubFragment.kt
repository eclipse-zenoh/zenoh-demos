package com.example.zenohapp.ui.examples

import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.sample.Sample
import io.zenoh.pubsub.Subscriber
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Dispatchers.Main
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class ZSubFragment : ZExampleFragment() {

    private var subscriber: Subscriber<Channel<Sample>>? = null

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        val session = viewModel.zenohSession!!
        val keyExpr = "demo/example/**".intoKeyExpr().getOrThrow()
        writeToConsole("Declaring Subscriber on '$keyExpr'...")
        subscriber = session.declareSubscriber(keyExpr, Channel()).getOrThrow()
        GlobalScope.launch(Dispatchers.IO) {
            val iterator = subscriber!!.receiver.iterator()
            while (iterator.hasNext()) {
                val sample = iterator.next()
                withContext(Main) {
                    writeToConsole(">> [Subscriber] Received ${sample.kind} ('${sample.keyExpr}': '${sample.payload}'" + "${
                        sample.attachment?.let {
                            ", with attachment: $it"
                        } ?: ""
                    })")
                }
            }
        }
    }

    override fun stopExample() {
        subscriber?.undeclare()
    }

    override fun onDestroyView() {
        super.onDestroyView()
        stopExample()
    }
}