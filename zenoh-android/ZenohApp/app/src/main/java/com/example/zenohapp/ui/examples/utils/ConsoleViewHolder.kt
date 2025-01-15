package com.example.zenohapp.ui.examples.utils

import android.view.View
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView.ViewHolder
import com.example.zenohapp.R

class ConsoleViewHolder(view: View): ViewHolder(view) {
    val entry: TextView = view.findViewById(R.id.entryTextView)
}