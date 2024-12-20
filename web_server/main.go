package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
)


func homeHandler(w http.ResponseWriter, r *http.Request) {
	if r.URL.Path != "/" {
		http.NotFound(w, r)
		return
	}
	http.ServeFile(w, r, "../web/templates/index.html")
}

func main() {
	host := "localhost"
	port := "1313"


	if len(os.Args) == 2 {
		host = os.Args[1]
	}

	fs := http.FileServer(http.Dir("../web/statics"))
	http.Handle("/statics/", http.StripPrefix("/statics/", fs))

	http.HandleFunc("/", homeHandler)


	addr := fmt.Sprintf("%s:%s", host, port)
	fmt.Printf("Serveur démarré sur http://%s\n", addr)

	err := http.ListenAndServe(addr, nil)
	if err != nil {
		log.Fatal("Erreur lors du démarrage du serveur : ", err)
	}
}
