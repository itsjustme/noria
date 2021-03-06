args <- commandArgs(trailingOnly = TRUE)
t <- data.frame()
for (a in args) {
	a <- strsplit(a, ":")
	label = a[[1]][[1]]
	file = a[[1]][[2]]
	v <- read.table(text = grep("percentile|avg", grep("PUT|GET", readLines(file), value=TRUE), invert=TRUE, value=TRUE))
	t <- rbind(t, data.frame(t=v[,1], op=sub("\\d?\\+?:$", "", v[,2]), opss=v[,3], variant=label, buffer=FALSE))
}
t$t <- t$t/1000000000.0
t$opss <- t$opss/1000.0

library(ggplot2)
p <- ggplot(data=t, aes(x=t, y=opss, color=variant))
p <- p + ylim(c(0,NA))
p <- p + facet_grid(op ~ buffer, scales="free_y")
p <- p + geom_point(size = 0.5, alpha = 0.5)# + geom_smooth()
p <- p + xlab("time") + ylab("kops/s") + ggtitle("Soup performance timeline")
ggsave('timeline.png',plot=p,width=10,height=6)
