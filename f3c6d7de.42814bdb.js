(window.webpackJsonp=window.webpackJsonp||[]).push([[41],{111:function(e,t,n){"use strict";n.r(t),n.d(t,"frontMatter",(function(){return i})),n.d(t,"metadata",(function(){return l})),n.d(t,"toc",(function(){return p})),n.d(t,"default",(function(){return c}));var a=n(3),r=n(7),o=(n(0),n(114)),i={id:"overview",title:"Overview"},l={unversionedId:"concepts/rpms/overview",id:"concepts/rpms/overview",isDocsHomePage:!1,title:"Overview",description:"Introduction to RPM",source:"@site/docs/concepts/rpms/overview.md",slug:"/concepts/rpms/overview",permalink:"/antlir/docs/concepts/rpms/overview",editUrl:"https://github.com/facebookincubator/antlir/edit/master/website/docs/concepts/rpms/overview.md",version:"current",sidebar:"docs",previous:{title:"Helper Buck Targets",permalink:"/antlir/docs/tutorials/helper-buck-targets"},next:{title:"How RPMs are Updated",permalink:"/antlir/docs/concepts/rpms/how-rpms-are-updated"}},p=[{value:"Introduction to RPM",id:"introduction-to-rpm",children:[]},{value:"Key concepts",id:"key-concepts",children:[{value:"Universe",id:"universe",children:[]},{value:"Package group",id:"package-group",children:[]},{value:"Repo snapshot",id:"repo-snapshot",children:[]},{value:"Repo snapshot debugging",id:"repo-snapshot-debugging",children:[]}]}],s={toc:p};function c(e){var t=e.components,n=Object(r.a)(e,["components"]);return Object(o.b)("wrapper",Object(a.a)({},s,n,{components:t,mdxType:"MDXLayout"}),Object(o.b)("h2",{id:"introduction-to-rpm"},"Introduction to RPM"),Object(o.b)("p",null,"The web has ample documentation on the ",Object(o.b)("a",Object(a.a)({parentName:"p"},{href:"https://www.redhat.com/sysadmin/how-manage-packages"}),"RedHat Package Manager ecosystem"),", so Antlir RPM docs\nfocus on Antlir-specific terminology and behaviors."),Object(o.b)("p",null,"To follow these docs, you'll want to be familiar with the following\nstandard terms:"),Object(o.b)("ul",null,Object(o.b)("li",{parentName:"ul"},Object(o.b)("p",{parentName:"li"},Object(o.b)("strong",{parentName:"p"},"Package"),": An archive file containing the files to be installed into\nthe OS + package manager metadata, including a list of dependencies.")),Object(o.b)("li",{parentName:"ul"},Object(o.b)("p",{parentName:"li"},Object(o.b)("strong",{parentName:"p"},"NEVRA"),': Name-Epoch-Version-Release-Architecture -- a unique ID for a\npackage within a "universe" (Antlir-specific, defined below).\nThe content of a NEVRA ',Object(o.b)("em",{parentName:"p"},"should")," be immutable (within a universe)."),Object(o.b)("p",{parentName:"li"},"Importantly, package managers define an old-to-new ordering on EVRAs,\nand OS management tools typically try to upgrade to the newest package\navailable."),Object(o.b)("p",{parentName:"li"},'Sometimes multiple packages share EVRA schemes, and even require\nmatching EVRAs across different package names -- see "package group"\nbelow.')),Object(o.b)("li",{parentName:"ul"},Object(o.b)("p",{parentName:"li"},Object(o.b)("strong",{parentName:"p"},"Repo"),': Short for "repository" -- a collection of packages, plus a set\nof indexes called ',Object(o.b)("strong",{parentName:"p"},"repodata")," (as XML or Sqlite databases) computed\nfrom those packages.  Typically hosted at an HTTP URL.  The root of the\nrepo is ",Object(o.b)("inlineCode",{parentName:"p"},"repomd.xml"),", which links to everything else."),Object(o.b)("p",{parentName:"li"},"A repo's content may change, as packages are added and removed.  As they\nevolve, repos generally attempt to maintain some kind of backwards\ncompatibility / upgrade path -- e.g.  CentOS8.1 should be upgradable to\nCentOS8.2 and so forth.  Not all repos have discrete point releases --\ne.g.  EPEL7 just moves forward continuously."),Object(o.b)("p",{parentName:"li"},"Note that the same package instance (or package name) can be contained\nin multiple repos, and the package manager will somehow pick one\n(implementation-defined behavior).  Therefore, it is important that\nrepos that are being used together be mutually compatible.")),Object(o.b)("li",{parentName:"ul"},Object(o.b)("p",{parentName:"li"},Object(o.b)("strong",{parentName:"p"},"Distro release"),': A collection of mutually compatible repositories. For\nexample, CentOS7.2 is a distro comprising multiple "standard" repos,\nwhile EPEL7 is an add-on repository intended to be compatible with\nall CentOS7.x distros.')),Object(o.b)("li",{parentName:"ul"},Object(o.b)("p",{parentName:"li"},Object(o.b)("strong",{parentName:"p"},"yum")," / ",Object(o.b)("strong",{parentName:"p"},"dnf"),": The package manager program, which installs packages\nand their dependencies into a filesystem root.")),Object(o.b)("li",{parentName:"ul"},Object(o.b)("p",{parentName:"li"},Object(o.b)("strong",{parentName:"p"},"{yum,dnf}.conf"),": Configuration for the package manager, including a\nlist of repos, plus install-time settings, like whether to install\noptional dependencies, or to validate package GPG signatures."))),Object(o.b)("h2",{id:"key-concepts"},"Key concepts"),Object(o.b)("h3",{id:"universe"},"Universe"),Object(o.b)("p",null,"An RPM-based operating system is typically built from a collection of\nmutually compatible repos.  In Antlir, each repo is assigned a ",Object(o.b)("strong",{parentName:"p"},"universe"),"\nname, which indicates the intended scope of mutual compatibility of packages\nwithin that repo."),Object(o.b)("p",null,"An OS may included repos may be larger than a single distro release, e.g.\nit is possible to have a structure like this:"),Object(o.b)("ul",null,Object(o.b)("li",{parentName:"ul"},'CentOS8.3 -- universe "centos8"'),Object(o.b)("li",{parentName:"ul"},'EPEL8 -- universe "centos8"'),Object(o.b)("li",{parentName:"ul"},'CentOS9Stream -- universe "centos9"'),Object(o.b)("li",{parentName:"ul"},'CompanyInternalRepo -- universe "company", statically linked, installed\nin ',Object(o.b)("inlineCode",{parentName:"li"},"/opt/company"),".")),Object(o.b)("p",null,"Key invariants for universes:"),Object(o.b)("ul",null,Object(o.b)("li",{parentName:"ul"},"All repos in a universe are mutually compatible."),Object(o.b)("li",{parentName:"ul"},'Additionally, some universes may also be mutally compatible. In the\nexample above, "centos8" and "centos9" are ',Object(o.b)("strong",{parentName:"li"},"not"),' mutually compatible.\nHowever, it is very reasonable for both "centos8" and "centos9" to be\ncompatible with "company".'),Object(o.b)("li",{parentName:"ul"},"For all repos in a universe, a package name must refer to the same\npiece of software (i.e. it must be upgradable)."),Object(o.b)("li",{parentName:"ul"},"Within a universe, a package NEVRA must uniquely identify the byte\ncontents of the package.  Caveat: if package re-signing is commonplace,\nwe may consider supporting an exemption in Antlir for ignoring\nthe signature when the package contents are otherwise identical.")),Object(o.b)("h3",{id:"package-group"},"Package group"),Object(o.b)("p",null,"Within a universe, a list of related packages, all of which must always have\nthe same installed version (and ought to be installed in one transaction).\nFor example, ",Object(o.b)("inlineCode",{parentName:"p"},"systemd"),", ",Object(o.b)("inlineCode",{parentName:"p"},"systemd-libs"),", and ",Object(o.b)("inlineCode",{parentName:"p"},"systemd-devel")," must be in sync.\nEach package may be in at most one package group."),Object(o.b)("h3",{id:"repo-snapshot"},"Repo snapshot"),Object(o.b)("p",null,"Given a collection of repos from mutually compatible universes, Antlir\nhas code (see ",Object(o.b)("inlineCode",{parentName:"p"},"antlir/rpm/snapshot_repos.py"),") to:"),Object(o.b)("ul",null,Object(o.b)("li",{parentName:"ul"},"Given a `{yum,dnf}.conf}, download each repo (atomically within a repo,\nbut not across repos)."),Object(o.b)("li",{parentName:"ul"},"Save the packages and repodata to append-only storage. This step\nuses a database (",Object(o.b)("inlineCode",{parentName:"li"},"antlir/rpm/repo_db.py"),") to avoid redundantly storing\nobjects that were already captured by a previous snapshot."),Object(o.b)("li",{parentName:"ul"},"Save a build-time index of all the packages in all the repos,\ncalled a ",Object(o.b)("inlineCode",{parentName:"li"},"RepoSnapshot")," (",Object(o.b)("inlineCode",{parentName:"li"},"antlir/rpm/repo_snapshot.py"),"). This is\nserialized to SQLite and also uploaded to append-only storage."),Object(o.b)("li",{parentName:"ul"},"Run ",Object(o.b)("inlineCode",{parentName:"li"},"nspawn_in_subvol")," containers, which are able to\n",Object(o.b)("strong",{parentName:"li"},"deterministically")," install RPMs from a ",Object(o.b)("inlineCode",{parentName:"li"},"RepoSnapshot"),".  This is\nnormally arranged by committing to source control:",Object(o.b)("ul",{parentName:"li"},Object(o.b)("li",{parentName:"ul"},"The storage ID of the ",Object(o.b)("inlineCode",{parentName:"li"},"RepoSnapshot")),Object(o.b)("li",{parentName:"ul"},Object(o.b)("inlineCode",{parentName:"li"},"{yum,dnf}.conf")," corresponding to the snapshot"),Object(o.b)("li",{parentName:"ul"},"Trusted GPG keys")))),Object(o.b)("h3",{id:"repo-snapshot-debugging"},"Repo snapshot debugging"),Object(o.b)("p",null,"See commands for ",Object(o.b)("a",Object(a.a)({parentName:"p"},{href:"/antlir/docs/faq#how-do-i-inspect-the-rpm-snapshot-db"}),"inspecting RPM snapshots in the FAQ"),"."))}c.isMDXComponent=!0},114:function(e,t,n){"use strict";n.d(t,"a",(function(){return b})),n.d(t,"b",(function(){return d}));var a=n(0),r=n.n(a);function o(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}function i(e,t){var n=Object.keys(e);if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);t&&(a=a.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),n.push.apply(n,a)}return n}function l(e){for(var t=1;t<arguments.length;t++){var n=null!=arguments[t]?arguments[t]:{};t%2?i(Object(n),!0).forEach((function(t){o(e,t,n[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(n)):i(Object(n)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(n,t))}))}return e}function p(e,t){if(null==e)return{};var n,a,r=function(e,t){if(null==e)return{};var n,a,r={},o=Object.keys(e);for(a=0;a<o.length;a++)n=o[a],t.indexOf(n)>=0||(r[n]=e[n]);return r}(e,t);if(Object.getOwnPropertySymbols){var o=Object.getOwnPropertySymbols(e);for(a=0;a<o.length;a++)n=o[a],t.indexOf(n)>=0||Object.prototype.propertyIsEnumerable.call(e,n)&&(r[n]=e[n])}return r}var s=r.a.createContext({}),c=function(e){var t=r.a.useContext(s),n=t;return e&&(n="function"==typeof e?e(t):l(l({},t),e)),n},b=function(e){var t=c(e.components);return r.a.createElement(s.Provider,{value:t},e.children)},m={inlineCode:"code",wrapper:function(e){var t=e.children;return r.a.createElement(r.a.Fragment,{},t)}},u=r.a.forwardRef((function(e,t){var n=e.components,a=e.mdxType,o=e.originalType,i=e.parentName,s=p(e,["components","mdxType","originalType","parentName"]),b=c(n),u=a,d=b["".concat(i,".").concat(u)]||b[u]||m[u]||o;return n?r.a.createElement(d,l(l({ref:t},s),{},{components:n})):r.a.createElement(d,l({ref:t},s))}));function d(e,t){var n=arguments,a=t&&t.mdxType;if("string"==typeof e||a){var o=n.length,i=new Array(o);i[0]=u;var l={};for(var p in t)hasOwnProperty.call(t,p)&&(l[p]=t[p]);l.originalType=e,l.mdxType="string"==typeof e?e:a,i[1]=l;for(var s=2;s<o;s++)i[s]=n[s];return r.a.createElement.apply(null,i)}return r.a.createElement.apply(null,n)}u.displayName="MDXCreateElement"}}]);