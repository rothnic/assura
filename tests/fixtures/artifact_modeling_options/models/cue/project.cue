package artifacts

#Goal: {
	id:     string & !=""
	title:  string & !=""
	status: "active" | "completed"
	owners: [string & !="", ...(string & !"")]
	specs?: [...string]
}

#Spec: {
	id:     string & !=""
	title:  string & !=""
	status: "draft" | "active"
}

#Decision: {
	id:         string & !=""
	title:      string & !=""
	status:     "active" | "superseded"
	supersedes?: string
}
