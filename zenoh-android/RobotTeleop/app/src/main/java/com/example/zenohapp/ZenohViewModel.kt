package com.example.zenohapp

import androidx.lifecycle.ViewModel
import io.zenoh.Session

class ZenohViewModel : ViewModel() {

    var zenohSession: Session? = null

}