package com.example.zenohapp.ui.examples

import io.zenoh.bytes.ZBytes
import io.zenoh.keyexpr.KeyExpr
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.query.Query
import io.zenoh.query.Queryable
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Dispatchers.Main
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.apache.commons.net.ntp.TimeStamp

class ZQueryableFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZQueryableFragment::javaClass.name
    }

    private var queryable: Queryable<Channel<Query>>? = null

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        val session = viewModel.zenohSession!!
        val keyExpr = "demo/example/zenoh-android-queryable".intoKeyExpr().getOrThrow()
        writeToConsole("Declaring Queryable on '$keyExpr'...\n")
        session.declareQueryable(keyExpr, channel = Channel()).onSuccess {
            queryable = it
            GlobalScope.launch(Dispatchers.IO) {
                it.receiver.apply {
                    handleRequests(this, keyExpr)
                }
            }
        }.onFailure {
            handleError(TAG, "Failed to launch queryable", it)
        }

    }

    override fun stopExample() {
        queryable?.undeclare()
    }

    override fun onDestroyView() {
        super.onDestroyView()
        queryable?.close()
    }

    private suspend fun handleRequests(
        receiverChannel: Channel<Query>, keyExpr: KeyExpr
    ) {
        val iterator = receiverChannel.iterator()
        while (iterator.hasNext()) {
            iterator.next().use { query ->
                val valueInfo = query.payload?.let { value -> " with value '$value'" } ?: ""
                withContext(Main) {
                    writeToConsole(">> [Queryable] Received Query '${query.selector}' $valueInfo")
                }
                query.reply(
                    keyExpr,
                    payload = ZBytes.from("Queryable from Android"),
                    timestamp = TimeStamp.getCurrentTime()
                )
                    .onFailure { withContext(Main) { writeToConsole(">> [Queryable ] Error sending reply: $it\n") } }
            }
        }
    }
}