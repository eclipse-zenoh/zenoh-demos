package com.example.zenohapp.ui.examples

import android.util.Log
import io.zenoh.keyexpr.KeyExpr
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.subscriber.Subscriber
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Dispatchers.Main
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext


class ZSubThrFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZSubFragment::javaClass.name
    }

    private var subscriber: Subscriber<Unit>? = null
    private var keyExpr: KeyExpr? = null

    val NANOS_TO_SEC = 1_000_000_000L
    val n = 50000L
    var batchCount = 0
    var count = 0
    var startTimestampNs: Long = 0
    var globalStartTimestampNs: Long = 0

    fun listener() {
        if (count == 0) {
            startTimestampNs = System.nanoTime()
            if (globalStartTimestampNs == 0L) {
                globalStartTimestampNs = startTimestampNs
            }
            count++
            return
        }
        if (count < n) {
            count++
            return
        }
        val stop = System.nanoTime()
        val msgs = n * NANOS_TO_SEC / (stop - startTimestampNs)
        GlobalScope.launch(Dispatchers.IO) {
            withContext(Main) {
                Log.i(TAG,"$msgs msgs/sec")
                console.append("$msgs msgs/sec\n")
            }
        }
        batchCount++
        count = 0
    }

    fun report() {
        val end = System.nanoTime()
        val total = batchCount * n + count
        val msgs = (end - globalStartTimestampNs) / NANOS_TO_SEC
        val avg = total * NANOS_TO_SEC / (end - globalStartTimestampNs)
        GlobalScope.launch(Dispatchers.IO) {
            withContext(Main) {
                console.append("Received $total messages in $msgs: averaged $avg msgs/sec\n")
            }
        }
    }

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        batchCount = 0
        count = 0
        startTimestampNs = 0
        globalStartTimestampNs = 0
        viewModel.zenohSession?.apply {
            "test/thr".intoKeyExpr().onSuccess { key ->
                keyExpr = key
                console.append("Declaring Subscriber on '$key'...\n")
                this.declareSubscriber(key).reliable().with { listener() }.res().onSuccess { sub ->
                    subscriber = sub
                    GlobalScope.launch(Dispatchers.IO) {
                        withContext(Main) {
                            console.append("Launched...")
                        }
                    }
                }.onFailure {
                    handleError(TAG, "Failed to launch subscriber", it)
                }
            }
        }
    }

    override fun stopExample() {
        report()
        subscriber?.undeclare()
        subscriber = null
        keyExpr?.close()
    }

    override fun onDestroyView() {
        super.onDestroyView()
        stopExample()
    }
}