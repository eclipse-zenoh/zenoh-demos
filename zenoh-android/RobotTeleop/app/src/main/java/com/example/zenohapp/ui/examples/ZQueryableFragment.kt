package com.example.zenohapp.ui.examples

import io.zenoh.keyexpr.KeyExpr
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.prelude.SampleKind
import io.zenoh.queryable.Query
import io.zenoh.queryable.Queryable
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
    private var keyExpr: KeyExpr? = null

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        viewModel.zenohSession?.apply {
            "demo/example/zenoh-android-queryable".intoKeyExpr().onSuccess { key ->
                keyExpr = key
                console.append("Declaring Queryable on '$key'...\n")
                this.declareQueryable(key).res().onSuccess {
                    queryable = it
                    GlobalScope.launch(Dispatchers.IO) {
                        it.receiver?.apply {
                            handleRequests(this, key)
                        }
                    }
                }.onFailure {
                    handleError(TAG, "Failed to launch queryable", it)
                }
            }
        }
    }

    override fun stopExample() {
        queryable?.undeclare()
    }

    override fun onDestroyView() {
        super.onDestroyView()
        keyExpr?.close()
        queryable?.close()
    }

    private suspend fun handleRequests(
        receiverChannel: Channel<Query>, keyExpr: KeyExpr
    ) {
        val iterator = receiverChannel.iterator()
        while (iterator.hasNext()) {
            iterator.next().use { query ->
                val valueInfo = query.value?.let { value -> " with value '$value'" } ?: ""
                withContext(Main) {
                    console.append(">> [Queryable] Received Query '${query.selector}' $valueInfo\n")
                }
                query.reply(keyExpr).success("Queryable from Android!").withKind(SampleKind.PUT)
                    .withTimeStamp(TimeStamp.getCurrentTime()).res()
                    .onFailure { withContext(Main) { console.append(">> [Queryable ] Error sending reply: $it\n") } }
            }
        }
    }
}