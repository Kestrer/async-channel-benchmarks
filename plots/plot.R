#!/bin/R -f

open.svg <- function(name) {
	svg(name, width = 16, height = 9)
	par(family = "sans-serif")
	par(cex.main = 2)
	par(cex.lab = 1.5)
}

channel.lty <- function(channel) {
	if (grepl("flume", channel, fixed = TRUE)) {
		return(5)
	}
	return(1)
}
channel.lwd <- function(channel) {
	if (grepl("unbounded", channel, fixed = TRUE)) {
		return(2)
	}
	return(1)
}

for (s in c("send", "recv")) {
	data <- read.csv(sprintf("../data/oneshot-%s.csv", s))
	data <- setNames(data$time, data$channel)

	open.svg(sprintf("oneshot-%s.svg", s))

	par(mar = c(15, 5, 4, 2))

	barplot(
		data,
		main = sprintf("Oneshot %s", s),
		ylab = "Time (ns)",
		ylim = c(0, 800),
		las = 2,
		mgp = c(3, 1, 0),
	)
	grid(ny = 8, nx = 0)

	dev.off()
}

for (s in c("send", "recv")) {
	data <- read.csv(sprintf("../data/mpmc-%s.csv", s), check.names = FALSE)

	open.svg(sprintf("mpmc-%s.svg", s))

	par(mar = c(5, 5, 4, 2))
	par(pch = 4)

	plot(
		x = c(0),
		xlim = c(0, 4),
		ylim = c(0, 40000),
		type = "n",
		main = sprintf("MPMC %s", s),
		xlab = "Contention (threads)",
		ylab = "Time (ns)",
	)

	colors <- sapply(
		1:ncol(data) - 1,
		function(i) {
			i <- i / (ncol(data) - 1)
			i <- i %% 0.5 * 2
			return(hsv(i * 3 / 4, 1, 0.9))
		}
	)

	i <- 1
	for (channel in colnames(data)[-1]) {
		lines(
			cbind(data$contention, data[[channel]]),
			type = "l",
			lty = channel.lty(channel),
			lwd = channel.lwd(channel),
			col = colors[i],
		)
		i <- i + 1
	}

	legend(
		0,
		40000,
		legend = colnames(data)[-1],
		col = colors,
		lty = sapply(colnames(data)[-1], channel.lty),
		lwd = sapply(colnames(data)[-1], channel.lwd),
	)

	dev.off()
}
warnings()
